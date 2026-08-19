#!/usr/bin/env python3
"""Validate release governance, publication eligibility, and live controls."""

from __future__ import annotations

import argparse
import json
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
WORKFLOW_CHECKER = (
    "cargo",
    "run",
    "--quiet",
    "--locked",
    "--manifest-path",
    "tools/prepared-coverage-check/Cargo.toml",
    "--bin",
    "workflow-policy-check",
    "--",
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


def check_workflows(paths: list[Path], *, root: Path = ROOT) -> None:
    result = subprocess.run(
        [*WORKFLOW_CHECKER, *(str(path) for path in paths)],
        cwd=root,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or "semantic workflow validation failed"
        raise GovernanceError(detail)


def check_workflow(path: Path) -> None:
    check_workflows([path])


def workflow_files(workflows: Path) -> dict[str, Path]:
    paths = {
        path.name: path
        for pattern in ("*.yml", "*.yaml")
        for path in workflows.glob(pattern)
        if path.is_file()
    }
    return paths


def check_workflow_inventory(workflows: Path, expected: set[str]) -> dict[str, Path]:
    discovered = workflow_files(workflows)
    if set(discovered) != expected:
        raise GovernanceError(
            "workflow inventory differs: "
            f"expected {sorted(expected)}, actual {sorted(discovered)}"
        )
    return discovered


def check_source_governance(root: Path, config: dict) -> None:
    workflows = root / ".github" / "workflows"
    expected = set(config["workflows"]["files"])
    discovered = check_workflow_inventory(workflows, expected)
    check_workflows([discovered[name] for name in sorted(expected)], root=root)

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


def normalized_bypass_actors(actors: list[dict]) -> tuple[tuple[object, ...], ...]:
    normalized = []
    for actor in actors:
        normalized.append(
            (
                actor.get("actor_type"),
                actor.get("actor_id"),
                actor.get("bypass_mode"),
            )
        )
    return tuple(sorted(normalized, key=repr))


def check_live_github(config: dict, query=gh_json) -> None:
    policy = config["policy"]
    repository = policy["repository"]
    repository_state = query(f"repos/{repository}")
    if repository_state.get("default_branch") != policy["default_branch"]:
        raise GovernanceError("default branch differs from reviewed policy")

    rulesets = query(f"repos/{repository}/rulesets")
    active = [item for item in rulesets if item.get("enforcement") == "active"]
    branch_rulesets = [
        item
        for item in active
        if item.get("target") == "branch"
        and item.get("name") == policy["branch_ruleset_name"]
    ]
    if len(branch_rulesets) != 1:
        raise GovernanceError("reviewed branch ruleset is not uniquely active")
    tag_rulesets = [item for item in active if item.get("target") == "tag"]
    if bool(tag_rulesets) != policy["tag_ruleset_required"]:
        raise GovernanceError("tag ruleset state differs from reviewed policy")

    details = query(f"repos/{repository}/rulesets/{branch_rulesets[0]['id']}")
    observed_bypasses = normalized_bypass_actors(details.get("bypass_actors", []))
    expected_bypasses = normalized_bypass_actors(
        policy["expected_branch_bypass_actors"]
    )
    if observed_bypasses != expected_bypasses:
        raise GovernanceError("branch ruleset bypass actors differ from reviewed policy")
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

    workflow = query(f"repos/{repository}/actions/permissions/workflow")
    if (
        workflow.get("default_workflow_permissions")
        != policy["workflow_default_permissions"]
    ):
        raise GovernanceError("GitHub Actions default token is not read-only")
    if (
        workflow.get("can_approve_pull_request_reviews")
        != policy["workflow_can_approve_pull_requests"]
    ):
        raise GovernanceError("Actions review approval differs from reviewed policy")

    codeql = query(f"repos/{repository}/code-scanning/default-setup")
    if codeql.get("state") != policy["codeql_state"]:
        raise GovernanceError("CodeQL state differs from reviewed policy")
    if codeql.get("query_suite") != policy["codeql_setup"]:
        raise GovernanceError("CodeQL query suite differs from reviewed policy")
    if codeql.get("schedule") != policy["codeql_schedule"]:
        raise GovernanceError("CodeQL schedule differs from reviewed policy")
    if set(codeql.get("languages", [])) != set(policy["codeql_languages"]):
        raise GovernanceError("CodeQL languages differ from reviewed policy")

    action_policy = query(f"repos/{repository}/actions/permissions")
    expected_action_policy = {
        "enabled": policy["actions_enabled"],
        "allowed_actions": policy["allowed_actions"],
        "sha_pinning_required": policy["sha_pinning_required"],
    }
    for key, expected in expected_action_policy.items():
        if action_policy.get(key) != expected:
            raise GovernanceError(f"Actions {key} differs from reviewed policy")

    security = repository_state.get("security_and_analysis", {})
    for key in ("secret_scanning", "secret_scanning_push_protection"):
        if security.get(key, {}).get("status") != policy[key]:
            raise GovernanceError(f"{key} differs from reviewed policy")
    print("Live GitHub release controls match the reviewed governance baseline.")


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
