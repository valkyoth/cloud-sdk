#!/usr/bin/env python3
"""Validate release governance, publication eligibility, and live controls."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - release host guard.
    print("Python 3.11+ is required because this script uses tomllib.", file=sys.stderr)
    raise

ROOT = Path(__file__).resolve().parents[1]
CONFIG = ROOT / "release-governance.toml"
PINNED_ACTION = re.compile(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_./-]+@[0-9a-f]{40}$")
FORBIDDEN_WORKFLOW_TEXT = (
    "cargo publish",
    "gh release",
    "id-token: write",
    "contents: write",
    "packages: write",
    "pull_request_target:",
)
REQUIRED_GOVERNANCE_TEXT = (
    "## Signer Lifecycle",
    "Normal rotation:",
    "Suspected compromise:",
    "## Ownership And Repository Recovery",
    "## Rollback And Incident Response",
    "## Review Independence",
    "not organizationally independent",
)
FORBIDDEN_PUBLISHER_COMMANDS = ("git push", "gh release", "cargo owner")


class GovernanceError(RuntimeError):
    """A release governance invariant was not satisfied."""


def load_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def capture_json(command: list[str], *, root: Path = ROOT) -> dict | list:
    raw = subprocess.check_output(command, cwd=root, text=True)
    return json.loads(raw)


def manifest_package(path: Path) -> tuple[str, object] | None:
    in_package = False
    name: str | None = None
    publish: object = None
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if line == "[package]":
            in_package = True
            continue
        if in_package and line.startswith("["):
            break
        if not in_package or not line or line.startswith("#"):
            continue
        if line.startswith("name = "):
            name = line.removeprefix("name = ").strip().strip('"')
        elif line == "publish = false":
            publish = False
        elif line.startswith("publish = "):
            publish = line.removeprefix("publish = ").strip()
    if not in_package:
        return None
    if not name:
        raise GovernanceError(f"{path}: package name must be nonempty text")
    return name, publish


def package_manifests(root: Path) -> dict[str, tuple[Path, object]]:
    packages: dict[str, tuple[Path, object]] = {}
    for path in sorted(root.rglob("Cargo.toml")):
        if any(part in {".git", "target"} for part in path.parts):
            continue
        package = manifest_package(path)
        if package is None:
            continue
        name, publish = package
        if name in packages:
            raise GovernanceError(f"duplicate package name {name}")
        packages[name] = (path, publish)
    return packages


def check_package_policy(root: Path, config: dict) -> None:
    package_config = config["packages"]
    publishable = tuple(package_config["publishable"])
    excluded = tuple(package_config["excluded"])
    if len(set(publishable)) != len(publishable):
        raise GovernanceError("publishable package allowlist contains duplicates")
    if set(publishable) & set(excluded):
        raise GovernanceError("publishable and excluded package sets overlap")

    manifests = package_manifests(root)
    expected = set(publishable) | set(excluded)
    if set(manifests) != expected:
        raise GovernanceError(
            "package governance inventory differs: "
            f"expected {tuple(sorted(expected))}, actual {tuple(sorted(manifests))}"
        )
    for name in publishable:
        path, setting = manifests[name]
        if setting is False or setting == []:
            raise GovernanceError(f"{path}: publishable package {name} is disabled")
    for name in excluded:
        path, setting = manifests[name]
        if setting is not False:
            raise GovernanceError(
                f"{path}: excluded package {name} must set publish = false"
            )

    sys.path.insert(0, str(root / "scripts"))
    import release_crates  # pylint: disable=import-outside-toplevel

    if tuple(release_crates.PUBLISH_ORDER) != publishable:
        raise GovernanceError("publisher order differs from governance allowlist")
    plan = release_crates.release_plan(root / "release-crates.toml")
    selected = release_crates.publish_plan(plan)
    if not set(selected) <= set(publishable):
        raise GovernanceError("release plan selected a package outside the allowlist")
    if plan["stage"] == "internal" and selected:
        raise GovernanceError("internal milestone selected packages for publication")


def top_level_permissions(lines: list[str], path: Path) -> dict[str, str]:
    indices = [index for index, line in enumerate(lines) if line == "permissions:"]
    if len(indices) != 1:
        raise GovernanceError(f"{path}: requires one top-level permissions block")
    permissions: dict[str, str] = {}
    for line in lines[indices[0] + 1 :]:
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        indentation = len(line) - len(line.lstrip())
        if indentation == 0:
            break
        if indentation != 2 or ":" not in line:
            raise GovernanceError(f"{path}: malformed permissions entry {line!r}")
        key, value = line.strip().split(":", 1)
        permissions[key] = value.strip()
    return permissions


def check_workflow(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    if top_level_permissions(lines, path) != {"contents": "read"}:
        raise GovernanceError(f"{path}: workflow permissions must be contents: read")
    for line in lines:
        stripped = line.strip()
        if stripped == "permissions:" and line != "permissions:":
            raise GovernanceError(f"{path}: job-level permissions are forbidden")
        directive = stripped.removeprefix("- ").strip()
        if directive.startswith("uses:"):
            action = directive.removeprefix("uses:").strip().split(" #", 1)[0]
            if not PINNED_ACTION.fullmatch(action):
                raise GovernanceError(f"{path}: action is not SHA-pinned: {action}")
    lowered = text.lower()
    for forbidden in FORBIDDEN_WORKFLOW_TEXT:
        if forbidden in lowered:
            raise GovernanceError(f"{path}: forbidden workflow capability {forbidden}")


def check_source_governance(root: Path, config: dict) -> None:
    workflows = root / ".github" / "workflows"
    expected = set(config["workflows"]["files"])
    actual = {path.name for path in workflows.glob("*.yml")}
    if actual != expected:
        raise GovernanceError(
            f"workflow inventory differs: expected {sorted(expected)}, actual {sorted(actual)}"
        )
    for name in sorted(expected):
        check_workflow(workflows / name)

    owner = config["ownership"]["code_owner"]
    codeowners = (root / ".github" / "CODEOWNERS").read_text(encoding="utf-8")
    if codeowners.strip() != f"* {owner}":
        raise GovernanceError("CODEOWNERS does not match release governance")
    security = (root / "SECURITY.md").read_text(encoding="utf-8")
    if "docs/RELEASE_GOVERNANCE.md" not in security:
        raise GovernanceError("SECURITY.md does not link release governance")
    check_governance_document(root / "docs" / "RELEASE_GOVERNANCE.md")
    policy = config["policy"]
    if policy.get("trusted_publishing") is not False:
        raise GovernanceError("trusted publishing requires a separately reviewed change")
    if policy.get("independent_pentest") is not False:
        raise GovernanceError("independent pentest must not be claimed")
    check_publisher_source(root / policy["publication_driver"])


def check_governance_document(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for required in REQUIRED_GOVERNANCE_TEXT:
        if required not in text:
            raise GovernanceError(f"{path}: missing governance procedure {required}")


def check_publisher_source(path: Path) -> None:
    text = path.read_text(encoding="utf-8").lower()
    for command in FORBIDDEN_PUBLISHER_COMMANDS:
        if command in text:
            raise GovernanceError(f"{path}: publisher contains forbidden command {command}")


def gh_json(endpoint: str) -> dict | list:
    return capture_json(["gh", "api", endpoint])


def check_live_github(config: dict) -> None:
    repository = config["policy"]["repository"]
    rulesets = gh_json(f"repos/{repository}/rulesets")
    active = [item for item in rulesets if item.get("enforcement") == "active"]
    branch_rulesets = [item for item in active if item.get("target") == "branch"]
    if not branch_rulesets:
        raise GovernanceError("GitHub has no active branch ruleset")
    details = gh_json(f"repos/{repository}/rulesets/{branch_rulesets[0]['id']}")
    include = details.get("conditions", {}).get("ref_name", {}).get("include", [])
    if "~DEFAULT_BRANCH" not in include:
        raise GovernanceError("active ruleset does not protect the default branch")
    rules = {rule.get("type"): rule for rule in details.get("rules", [])}
    required = {
        "creation",
        "deletion",
        "non_fast_forward",
        "required_linear_history",
        "required_signatures",
        "pull_request",
        "code_scanning",
        "update",
    }
    if not required <= set(rules):
        raise GovernanceError("active branch ruleset lacks required controls")
    pull_request = rules["pull_request"].get("parameters", {})
    if pull_request.get("required_approving_review_count", 0) < 1:
        raise GovernanceError("branch ruleset does not require review")
    if not pull_request.get("require_code_owner_review"):
        raise GovernanceError("branch ruleset does not require CODEOWNERS review")
    if not pull_request.get("dismiss_stale_reviews_on_push"):
        raise GovernanceError("branch ruleset does not dismiss stale reviews")
    if not pull_request.get("require_last_push_approval"):
        raise GovernanceError("branch ruleset does not require last-push approval")
    scanners = rules["code_scanning"].get("parameters", {}).get(
        "code_scanning_tools", []
    )
    if not any(
        tool.get("tool") == "CodeQL"
        and tool.get("security_alerts_threshold") == "all"
        and tool.get("alerts_threshold") == "all"
        for tool in scanners
    ):
        raise GovernanceError("branch ruleset does not require all CodeQL alerts")

    workflow = gh_json(f"repos/{repository}/actions/permissions/workflow")
    if workflow.get("default_workflow_permissions") != "read":
        raise GovernanceError("GitHub Actions default token is not read-only")
    if workflow.get("can_approve_pull_request_reviews"):
        raise GovernanceError("GitHub Actions may approve pull requests")
    codeql = gh_json(f"repos/{repository}/code-scanning/default-setup")
    if codeql.get("state") != "configured":
        raise GovernanceError("CodeQL default setup is not configured")
    if not {"actions", "python", "rust"} <= set(codeql.get("languages", [])):
        raise GovernanceError("CodeQL default setup lacks a required language")

    action_policy = gh_json(f"repos/{repository}/actions/permissions")
    if not action_policy.get("enabled"):
        raise GovernanceError("GitHub Actions is disabled")
    repository_state = gh_json(f"repos/{repository}")
    security = repository_state.get("security_and_analysis", {})

    tag_rulesets = [item for item in active if item.get("target") == "tag"]
    bypasses = details.get("bypass_actors", [])
    print(
        "live governance limitations: "
        f"tag rulesets={len(tag_rulesets)}, branch bypass actors={len(bypasses)}, "
        f"allowed Actions={action_policy.get('allowed_actions')}, "
        f"platform SHA pins={action_policy.get('sha_pinning_required')}, "
        f"secret scanning={security.get('secret_scanning', {}).get('status')}"
    )


def check_live_crates_io(config: dict) -> None:
    required = config["ownership"]["required_crates_io_owner"]
    for package in config["packages"]["publishable"]:
        output = subprocess.check_output(
            ["cargo", "owner", "--list", package], cwd=ROOT, text=True
        )
        owners = tuple(line.strip().split()[0] for line in output.splitlines() if line)
        if required not in owners:
            raise GovernanceError(f"{package}: required crates.io owner is absent")
        print(f"{package}: {len(owners)} crates.io owner(s) verified")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--live", action="store_true")
    args = parser.parse_args()
    try:
        config = load_toml(CONFIG)
        check_package_policy(ROOT, config)
        check_source_governance(ROOT, config)
        if args.live:
            check_live_github(config)
            check_live_crates_io(config)
    except (GovernanceError, OSError, KeyError, ValueError, subprocess.SubprocessError) as error:
        print(f"release governance: {error}", file=sys.stderr)
        return 1
    print("Release governance and publication boundaries match policy.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
