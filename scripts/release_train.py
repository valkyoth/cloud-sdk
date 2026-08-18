#!/usr/bin/env python3
"""Validate staged releases and cumulative multi-crate changes."""

from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
STAGES = ("internal", "public")
CADENCE_BASELINE = (0, 50, 0)
EXTRA_PUBLIC_CHECKPOINTS = {(0, 99, 0)}


def publication_allowed(plan: dict) -> bool:
    return plan["stage"] == "public"


def parse_version(version: str) -> tuple[int, int, int]:
    parts = version.split(".")
    if len(parts) != 3:
        raise RuntimeError(f"version must be MAJOR.MINOR.PATCH: {version}")
    try:
        parsed = tuple(int(part) for part in parts)
    except ValueError as exc:
        raise RuntimeError(f"version must be numeric: {version}") from exc
    return parsed  # type: ignore[return-value]


def is_scheduled_checkpoint(version: str) -> bool:
    major, minor, patch = parse_version(version)
    parsed = (major, minor, patch)
    return parsed in EXTRA_PUBLIC_CHECKPOINTS or (
        major == 0 and patch == 0 and minor >= 50 and minor % 5 == 0
    )


def next_checkpoint(version: str) -> str:
    major, minor, _patch = parse_version(version)
    if major != 0:
        raise RuntimeError("development checkpoint calculation is pre-1.0 only")
    if 95 <= minor < 99:
        return "0.99.0"
    next_minor = ((minor // 5) + 1) * 5
    if next_minor >= 100:
        return "1.0.0"
    return f"0.{next_minor}.0"


def validate_release_context(release: dict) -> dict:
    version = release.get("version")
    milestone = release.get("milestone")
    baseline = release.get("baseline")
    review_baseline = release.get("review_baseline")
    milestones = release.get("cumulative_milestones")
    stage = release.get("stage")
    exceptional = release.get("exceptional")
    exception_reason = release.get("exception_reason")
    if not all(
        isinstance(value, str)
        for value in (
            version,
            milestone,
            baseline,
            review_baseline,
            stage,
            exception_reason,
        )
    ) or not isinstance(milestones, list) or not isinstance(exceptional, bool):
        raise RuntimeError("release train metadata is incomplete")
    if not all(isinstance(value, str) for value in milestones):
        raise RuntimeError("cumulative_milestones must contain versions")
    if stage not in STAGES:
        raise RuntimeError(f"release stage must be one of {STAGES}")
    if milestone != version:
        raise RuntimeError("release milestone must match release version")

    parsed = parse_version(version)
    parsed_baseline = parse_version(baseline)
    parsed_review_baseline = parse_version(review_baseline)
    if (
        parsed < CADENCE_BASELINE
        or parsed_baseline < CADENCE_BASELINE
        or parsed_review_baseline < CADENCE_BASELINE
    ):
        raise RuntimeError("staged release policy begins at v0.50.0")
    anchor = parsed == CADENCE_BASELINE and parsed_baseline == parsed
    if parsed_baseline > parsed or (parsed_baseline == parsed and not anchor):
        raise RuntimeError("release baseline must precede the milestone")
    if parsed_review_baseline > parsed or (
        parsed_review_baseline == parsed and not anchor
    ):
        raise RuntimeError("review baseline must precede the milestone")

    parsed_milestones = tuple(parse_version(value) for value in milestones)
    if len(set(parsed_milestones)) != len(parsed_milestones):
        raise RuntimeError("cumulative_milestones contains duplicates")
    if parsed_milestones != tuple(sorted(parsed_milestones)):
        raise RuntimeError("cumulative_milestones must be in version order")
    if anchor:
        if stage != "public" or milestones or exceptional:
            raise RuntimeError("v0.50.0 must remain the public cadence anchor")
    else:
        if not parsed_milestones or parsed_milestones[-1] != parsed:
            raise RuntimeError("cumulative_milestones must end at the milestone")
        if any(item <= parsed_baseline or item > parsed for item in parsed_milestones):
            raise RuntimeError("cumulative milestones must follow the public baseline")

    scheduled = is_scheduled_checkpoint(version)
    if stage == "internal":
        if parsed[0] != 0 or scheduled:
            raise RuntimeError("internal stage conflicts with release classification")
        if exceptional and not exception_reason.strip():
            raise RuntimeError("exceptional internal release requires a reason")
        if not exceptional and exception_reason:
            raise RuntimeError("ordinary internal release cannot have an exception reason")
    elif not anchor and parsed[0] == 0:
        if not scheduled and not exceptional:
            raise RuntimeError("off-cycle public releases must be exceptional")
        if exceptional and not exception_reason.strip():
            raise RuntimeError("exceptional public release requires a reason")
        if not exceptional and exception_reason:
            raise RuntimeError("non-exceptional release cannot have an exception reason")
    elif exceptional and not exception_reason.strip():
        raise RuntimeError("exceptional public release requires a reason")

    return {
        "version": version,
        "milestone": milestone,
        "baseline": baseline,
        "review_baseline": review_baseline,
        "cumulative_milestones": tuple(milestones),
        "stage": stage,
        "exceptional": exceptional,
        "exception_reason": exception_reason,
        "anchor": anchor,
    }


def semantic_tags_before(release: str) -> tuple[str, ...]:
    release_version = parse_version(release)
    raw = subprocess.check_output(
        ["git", "tag", "--list", "v*.*.*"], cwd=ROOT, text=True
    )
    parsed: list[tuple[int, int, int]] = []
    for tag in raw.splitlines():
        try:
            version = parse_version(tag.removeprefix("v"))
        except RuntimeError:
            continue
        if version < release_version:
            parsed.append(version)
    return tuple(".".join(str(part) for part in item) for item in sorted(parsed))


def validate_repository_train(plan: dict) -> None:
    if plan["anchor"]:
        return
    baseline = parse_version(plan["baseline"])
    expected = tuple(
        version
        for version in semantic_tags_before(plan["version"])
        if parse_version(version) > baseline
    ) + (plan["version"],)
    if plan["cumulative_milestones"] != expected:
        raise RuntimeError(
            "cumulative_milestones must list every tag after "
            f"v{plan['baseline']} through {plan['version']}: expected {expected}"
        )
    tags = semantic_tags_before(plan["version"])
    expected_review_baseline = tags[-1] if tags else plan["baseline"]
    if plan["review_baseline"] != expected_review_baseline:
        raise RuntimeError(
            "review_baseline must be the immediately preceding release tag: "
            f"expected {expected_review_baseline}, actual {plan['review_baseline']}"
        )


def validate_facade_previous_version(plan: dict) -> None:
    if plan["anchor"]:
        return
    if plan["stage"] == "public":
        expected = plan["baseline"]
    else:
        tags = semantic_tags_before(plan["version"])
        expected = tags[-1] if tags else plan["baseline"]
    actual = plan["crates"]["cloud-sdk"]["previous_version"]
    if actual != expected:
        raise RuntimeError(
            "cloud-sdk previous_version does not match the release train: "
            f"expected {expected}, actual {actual}"
        )


def changed_packages(packages: dict[str, dict], baseline: str) -> set[str]:
    tag = f"v{baseline}"
    if subprocess.run(
        ["git", "rev-parse", "-q", "--verify", f"refs/tags/{tag}"],
        cwd=ROOT,
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode != 0:
        raise RuntimeError(f"release baseline tag is missing: {tag}")
    changed: set[str] = set()
    for name, package in packages.items():
        relative = Path(package["manifest_path"]).resolve().parent.relative_to(ROOT)
        tracked = subprocess.check_output(
            ["git", "diff", "--name-only", tag, "--", str(relative)],
            cwd=ROOT,
            text=True,
        ).strip()
        untracked = subprocess.check_output(
            ["git", "ls-files", "--others", "--exclude-standard", "--", str(relative)],
            cwd=ROOT,
            text=True,
        ).strip()
        if tracked or untracked:
            changed.add(name)
    return changed


def validate_cumulative_package_changes(packages: dict[str, dict], plan: dict) -> None:
    if plan["stage"] != "public" or plan["anchor"]:
        return
    for name in changed_packages(packages, plan["baseline"]):
        if plan["crates"][name]["change"] == "unchanged":
            raise RuntimeError(
                f"{name} changed after v{plan['baseline']} but is marked unchanged"
            )
    validate_dependency_closure(packages, plan)


def validate_dependency_closure(packages: dict[str, dict], plan: dict) -> None:
    changed_versions = {
        name
        for name, entry in plan["crates"].items()
        if entry["version"] != entry["previous_version"]
    }
    for name, package in packages.items():
        if plan["crates"][name]["change"] != "unchanged":
            continue
        changed_dependencies = sorted(
            dependency["name"]
            for dependency in package["dependencies"]
            if dependency["name"] in changed_versions
        )
        if changed_dependencies:
            raise RuntimeError(
                f"{name} depends on changed internal packages but is marked "
                f"unchanged: {tuple(changed_dependencies)}"
            )
