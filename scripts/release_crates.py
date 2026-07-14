#!/usr/bin/env python3
"""Publish cloud-sdk workspace crates in crates.io dependency order."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - release host guard.
    print("Python 3.11+ is required because this script uses tomllib.", file=sys.stderr)
    raise


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PLAN = ROOT / "release-crates.toml"
CHANGE_KINDS = ("code", "dependency", "metadata", "unchanged")
RETIRED_PACKAGES = frozenset(
    {
        "cloud-sdk-hetzner-reqwest",
        "cloud-sdk-hetzner-sanitization",
        "cloud-sdk-hetzner-testkit",
    }
)

PUBLISH_ORDER = (
    "cloud-sdk",
    "cloud-sdk-sanitization",
    "cloud-sdk-reqwest",
    "cloud-sdk-testkit",
    "cloud-sdk-hetzner",
)


def run(command: list[str], *, dry_run: bool) -> None:
    print(f"+ {' '.join(command)}", flush=True)
    if dry_run:
        return
    subprocess.run(command, cwd=ROOT, check=True)


def capture(command: list[str]) -> str:
    return subprocess.check_output(command, cwd=ROOT, text=True).strip()


def try_capture(command: list[str]) -> str | None:
    result = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    if result.returncode != 0:
        return None
    return result.stdout.strip()


def load_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def parse_version(version: str) -> tuple[int, int, int]:
    parts = version.split(".")
    if len(parts) != 3:
        raise RuntimeError(f"version must be MAJOR.MINOR.PATCH: {version}")
    try:
        major, minor, patch = (int(part) for part in parts)
    except ValueError as exc:
        raise RuntimeError(f"version must be numeric: {version}") from exc
    return (major, minor, patch)


def previous_release_version(tags: tuple[str, ...], release: str) -> str:
    release_parts = parse_version(release)
    prior: list[tuple[int, int, int]] = []
    for tag in tags:
        if not tag.startswith("v"):
            continue
        try:
            candidate = parse_version(tag.removeprefix("v"))
        except RuntimeError:
            continue
        if candidate < release_parts:
            prior.append(candidate)
    if not prior:
        return "none"
    major, minor, patch = max(prior)
    return f"{major}.{minor}.{patch}"


def verify_facade_previous_version(plan: dict) -> None:
    raw_tags = capture(["git", "tag", "--list", "v*.*.*"])
    tags = tuple(line for line in raw_tags.splitlines() if line)
    expected = previous_release_version(tags, plan["version"])
    actual = plan["crates"]["cloud-sdk"]["previous_version"]
    if actual != expected:
        raise RuntimeError(
            "cloud-sdk previous_version does not match latest prior release tag: "
            f"expected {expected}, actual {actual}"
        )


def is_initial(previous: str) -> bool:
    return previous == "none"


def release_version(plan_path: Path = DEFAULT_PLAN) -> str:
    plan = load_toml(plan_path)
    return plan["release"]["version"]


def cargo_metadata() -> dict:
    raw = capture(["cargo", "metadata", "--format-version", "1", "--no-deps"])
    return json.loads(raw)


def workspace_packages(metadata: dict) -> dict[str, dict]:
    workspace_ids = set(metadata["workspace_members"])
    return {
        package["name"]: package
        for package in metadata["packages"]
        if package["id"] in workspace_ids
    }


def reject_retired_packages(package_names, *, source: str) -> None:
    retired = tuple(sorted(set(package_names) & RETIRED_PACKAGES))
    if retired:
        raise RuntimeError(f"{source} contains retired packages: {retired}")


def reject_nested_cloud_sdk_packages(package_names, *, source: str) -> None:
    prefix = "cloud-sdk-"
    nested = tuple(
        sorted(
            name
            for name in set(package_names)
            if name.startswith(prefix) and "-" in name.removeprefix(prefix)
        )
    )
    if nested:
        raise RuntimeError(
            f"{source} contains nested cloud-sdk packages; use one crate per "
            f"provider and provider-neutral boundaries: {nested}"
        )


def release_plan(plan_path: Path) -> dict:
    plan = load_toml(plan_path)
    release = plan.get("release", {})
    crates = plan.get("crates", {})
    version = release.get("version")
    if not isinstance(version, str):
        raise RuntimeError("release-crates.toml is missing [release].version")
    if release.get("policy") != "independent":
        raise RuntimeError("release-crates.toml must use policy = \"independent\"")
    reject_retired_packages(PUBLISH_ORDER, source="PUBLISH_ORDER")
    reject_retired_packages(crates, source="release-crates.toml")
    reject_nested_cloud_sdk_packages(PUBLISH_ORDER, source="PUBLISH_ORDER")
    reject_nested_cloud_sdk_packages(crates, source="release-crates.toml")
    if set(crates) != set(PUBLISH_ORDER):
        raise RuntimeError(
            "release-crates.toml crates are not in sync with PUBLISH_ORDER: "
            f"expected {tuple(sorted(PUBLISH_ORDER))}, actual {tuple(sorted(crates))}"
        )
    parse_version(version)
    for package_name, entry in crates.items():
        validate_plan_entry(package_name, entry, version)
    return {"version": version, "crates": crates}


def validate_plan_entry(package_name: str, entry: dict, release: str) -> None:
    previous = entry.get("previous_version")
    version = entry.get("version")
    change = entry.get("change")
    publish = entry.get("publish")
    reason = entry.get("reason")
    if not all(isinstance(value, str) for value in (previous, version, change, reason)):
        raise RuntimeError(f"{package_name} has incomplete release plan metadata")
    if change not in CHANGE_KINDS:
        raise RuntimeError(f"{package_name} has invalid change kind {change!r}")
    if not isinstance(publish, bool):
        raise RuntimeError(f"{package_name} publish must be true or false")

    planned_version = parse_version(version)
    release_parts = parse_version(release)

    if is_initial(previous):
        if change != "code":
            raise RuntimeError(f"{package_name} initial release must be a code change")
        if planned_version != release_parts:
            raise RuntimeError(
                f"{package_name} initial release version must be {release}"
            )
        if not publish:
            raise RuntimeError(f"{package_name} initial release must publish")
        return

    previous_version = parse_version(previous)
    if package_name == "cloud-sdk":
        if planned_version != release_parts:
            raise RuntimeError(
                f"{package_name} must always match release version {release}"
            )
        if not publish:
            raise RuntimeError(f"{package_name} must publish for every release")
        if change == "unchanged":
            raise RuntimeError(f"{package_name} cannot be unchanged in a release")

    if change == "code":
        if package_name == "cloud-sdk":
            expected = release_parts
            error = f"{package_name} has code changes, so version must be {release}"
        else:
            expected = (previous_version[0], previous_version[1] + 1, 0)
            error = (
                f"{package_name} has code changes, so independent crate "
                f"version must be {expected[0]}.{expected[1]}.{expected[2]}"
            )
        if planned_version != expected:
            raise RuntimeError(error)
        if not publish:
            raise RuntimeError(f"{package_name} has code changes but publish is false")
    elif change == "metadata":
        if planned_version != release_parts:
            raise RuntimeError(
                f"{package_name} has metadata changes, so version must be {release}"
            )
        if not publish:
            raise RuntimeError(
                f"{package_name} has metadata changes but publish is false"
            )
    elif change == "dependency":
        same_line = planned_version[:2] == previous_version[:2]
        patch_bump = planned_version[2] > previous_version[2]
        if not same_line or not patch_bump:
            raise RuntimeError(
                f"{package_name} dependency-only bumps must stay on the existing "
                "minor line and increase only the patch number"
            )
        if not publish:
            raise RuntimeError(
                f"{package_name} has dependency-only changes but publish is false"
            )
    else:
        if planned_version != previous_version:
            raise RuntimeError(
                f"{package_name} is unchanged but version differs from previous_version"
            )
        if publish:
            raise RuntimeError(f"{package_name} is unchanged but publish is true")


def require_clean_tree(*, allow_dirty: bool) -> None:
    if allow_dirty:
        return
    status = capture(["git", "status", "--porcelain"])
    if status:
        print("Refusing to publish from a dirty worktree:", file=sys.stderr)
        print(status, file=sys.stderr)
        print("Commit or stash changes before publishing.", file=sys.stderr)
        sys.exit(1)


def verify_publish_order(packages: dict[str, dict], plan: dict) -> None:
    reject_retired_packages(PUBLISH_ORDER, source="PUBLISH_ORDER")
    reject_retired_packages(packages, source="workspace metadata")
    reject_retired_packages(plan["crates"], source="release plan")
    reject_nested_cloud_sdk_packages(PUBLISH_ORDER, source="PUBLISH_ORDER")
    reject_nested_cloud_sdk_packages(packages, source="workspace metadata")
    reject_nested_cloud_sdk_packages(plan["crates"], source="release plan")
    expected_names = tuple(sorted(PUBLISH_ORDER))
    actual_names = tuple(sorted(packages))
    if actual_names != expected_names:
        raise RuntimeError(
            "release_crates.py PUBLISH_ORDER is not in sync with workspace "
            f"packages: expected {expected_names}, actual {actual_names}"
        )

    seen: set[str] = set()
    for package_name in PUBLISH_ORDER:
        package = packages[package_name]
        planned_version = plan["crates"][package_name]["version"]
        if package["version"] != planned_version:
            raise RuntimeError(
                f"{package_name} is version {package['version']}, "
                f"expected {planned_version}"
            )

        for dependency in package["dependencies"]:
            dependency_name = dependency["name"]
            if dependency_name in packages and dependency_name not in seen:
                raise RuntimeError(
                    f"{package_name} depends on {dependency_name}, but "
                    f"{dependency_name} appears later in PUBLISH_ORDER"
                )
        seen.add(package_name)


def check_release_tag(version: str, *, require_tag: bool) -> None:
    tag = f"v{version}"
    head = try_capture(["git", "rev-parse", "HEAD"])
    tagged_commit = try_capture(["git", "rev-list", "-n", "1", tag])
    if head is None or tagged_commit is None:
        message = f"release tag {tag!r} was not found"
        if require_tag:
            print(f"Refusing to publish: {message}.", file=sys.stderr)
            sys.exit(1)
        print(f"Warning: {message}.", file=sys.stderr)
        return

    if head != tagged_commit:
        message = f"HEAD is not tagged as {tag} (HEAD {head}, {tag} {tagged_commit})"
        if require_tag:
            print(f"Refusing to publish: {message}.", file=sys.stderr)
            sys.exit(1)
        print(f"Warning: {message}.", file=sys.stderr)
        return

    tag_type = try_capture(["git", "cat-file", "-t", tag])
    signature = try_capture(["git", "verify-tag", tag])
    if tag_type != "tag" or signature is None:
        message = f"release tag {tag} is not an annotated, verifiable signed tag"
        if require_tag:
            print(f"Refusing to publish: {message}.", file=sys.stderr)
            sys.exit(1)
        print(f"Warning: {message}.", file=sys.stderr)
        return

    print(f"Signed release tag {tag} points at HEAD and verifies.")


def run_preflight(args: argparse.Namespace) -> None:
    if not args.rerun_gate:
        print(
            "Release gate not rerun: the signed tag binds the pre-tag gate and "
            "GitHub-approved commit."
        )
        return

    version = parse_version(args.version)
    gate = ROOT / "scripts" / f"release_{version[0]}_{version[1]}_gate.sh"
    if gate.exists():
        run([str(gate.relative_to(ROOT))], dry_run=args.dry_run)
    else:
        run(["scripts/checks.sh"], dry_run=args.dry_run)


def publish_plan(plan: dict) -> tuple[str, ...]:
    reject_retired_packages(PUBLISH_ORDER, source="PUBLISH_ORDER")
    reject_retired_packages(plan["crates"], source="release plan")
    reject_nested_cloud_sdk_packages(PUBLISH_ORDER, source="PUBLISH_ORDER")
    reject_nested_cloud_sdk_packages(plan["crates"], source="release plan")
    return tuple(
        package for package in PUBLISH_ORDER if plan["crates"][package]["publish"]
    )


def selected_steps(start_at: str, steps: tuple[str, ...]) -> tuple[str, ...]:
    if not steps:
        return ()
    try:
        index = steps.index(start_at)
    except ValueError as exc:
        raise RuntimeError(f"unknown package for --start-at: {start_at}") from exc
    return steps[index:]


def wait_for_index(package: str, version: str, *, dry_run: bool) -> None:
    print()
    print(f"Published {package} {version}.")
    print(f"Wait until crates.io shows: https://crates.io/crates/{package}/{version}")
    print("Then press Enter to continue with dependent crates.")
    if dry_run:
        print("[dry-run] skipping wait")
        return
    input()
    time.sleep(5)


def publish(package: str, args: argparse.Namespace) -> None:
    reject_retired_packages((package,), source="publish request")
    reject_nested_cloud_sdk_packages((package,), source="publish request")
    command = ["cargo", "publish", "-p", package]
    run(command, dry_run=args.dry_run)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Publish cloud-sdk workspace crates in crates.io order."
    )
    parser.add_argument("--version", default=None)
    parser.add_argument("--plan", default=str(DEFAULT_PLAN))
    parser.add_argument("--start-at", default=None, choices=PUBLISH_ORDER)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--yes", action="store_true")
    parser.add_argument(
        "--rerun-gate",
        action="store_true",
        help="Rerun the full version gate after tag verification.",
    )
    args = parser.parse_args()

    raw_plan_path = Path(args.plan)
    plan_path = raw_plan_path if raw_plan_path.is_absolute() else (ROOT / raw_plan_path)
    plan = release_plan(plan_path.resolve())
    if plan_path.resolve() == DEFAULT_PLAN.resolve():
        verify_facade_previous_version(plan)
    if args.version is None:
        args.version = plan["version"]
    elif args.version != plan["version"]:
        print(
            f"Refusing to publish: --version {args.version} does not match "
            f"{plan_path.name} release {plan['version']}.",
            file=sys.stderr,
        )
        return 1

    packages = workspace_packages(cargo_metadata())
    verify_publish_order(packages, plan)

    if args.check:
        print("release_crates.py publish order is up to date.")
        print(f"release_crates.py release plan is {args.version}.")
        return 0

    require_clean_tree(allow_dirty=args.dry_run)
    check_release_tag(args.version, require_tag=not args.dry_run)
    run(
        ["scripts/validate-release-metadata.sh", "--release"],
        dry_run=args.dry_run,
    )

    planned_publish = publish_plan(plan)
    start_at = args.start_at or (planned_publish[0] if planned_publish else "")
    steps = selected_steps(start_at, planned_publish)

    print(f"Workspace root: {ROOT}")
    print(f"Release version: {args.version}")
    print("Publish sequence:")
    for package in steps:
        version = plan["crates"][package]["version"]
        change = plan["crates"][package]["change"]
        print(f"  - {package} {version} ({change})")
    if not steps:
        print("  - no crates selected for publishing")
    print()

    run_preflight(args)

    if not args.yes:
        answer = input("Type the release version to start publishing: ").strip()
        if answer != args.version:
            print("Version confirmation did not match; aborting.", file=sys.stderr)
            return 1

    for index, package in enumerate(steps):
        publish(package, args)
        version = plan["crates"][package]["version"]
        if index != len(steps) - 1:
            wait_for_index(package, version, dry_run=args.dry_run)

    print()
    print("Release publish sequence completed.")
    print(f"Recommended follow-up: cargo info cloud-sdk@{args.version}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except subprocess.CalledProcessError as error:
        command = " ".join(str(part) for part in error.cmd)
        print(
            f"Release command failed with status {error.returncode}: {command}",
            file=sys.stderr,
        )
        raise SystemExit(error.returncode) from None
