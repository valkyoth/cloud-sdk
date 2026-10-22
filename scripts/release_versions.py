"""Per-package version transitions for independent release planning."""

from release_train import parse_version

CHANGE_KINDS = ("code", "dependency", "metadata", "unchanged")


def is_initial(previous: str) -> bool:
    return previous == "none"


def validate_plan_entry(
    package_name: str,
    entry: dict,
    release: str,
    stage: str = "public",
) -> None:
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

    if stage == "candidate":
        if publish:
            raise RuntimeError(f"{package_name} cannot publish at candidate stage")
        if planned_version != release_parts:
            raise RuntimeError(
                f"{package_name} must match candidate version {release}"
            )
        if is_initial(previous):
            if change != "code":
                raise RuntimeError(
                    f"{package_name} initial candidate must be a code change"
                )
        elif change not in {"code", "metadata"}:
            raise RuntimeError(
                f"{package_name} candidate change must be code or metadata"
            )
        return

    if stage == "internal":
        if publish:
            raise RuntimeError(f"{package_name} cannot publish at internal stage")
        if package_name == "cloud-sdk":
            if planned_version != release_parts:
                raise RuntimeError(
                    f"{package_name} must always match release version {release}"
                )
            if change == "unchanged":
                raise RuntimeError(f"{package_name} cannot be unchanged in a release")
        elif not is_initial(previous) and planned_version != parse_version(previous):
            raise RuntimeError(
                f"{package_name} must retain its published version at internal stage"
            )
        return

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
            if (release_parts[0] >= 1 and release_parts[2] > 0
                    and previous_version[:2] == release_parts[:2]):
                expected = (*previous_version[:2], previous_version[2] + 1)
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
