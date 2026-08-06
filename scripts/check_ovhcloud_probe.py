#!/usr/bin/env python3
"""Check the source-locked OVHcloud probe and its no-publish boundary."""

from __future__ import annotations

import csv
import json
import sys
from pathlib import Path

from ovhcloud_probe_adapter import CANDIDATE_PATHS
from provider_drift_model import (
    read_bounded_json,
    validate_lock,
    validate_observation,
    validate_plugin,
)
from provider_drift_report import build_report
import release_crates


ROOT = Path(__file__).resolve().parents[1]
PROBE = ROOT / "provider-probes" / "ovhcloud-v2"
FIELDS = ("id", "method", "path", "pagination", "response_type", "stability")


def read_inventory(path: Path) -> list[dict[str, str]]:
    try:
        with path.open("r", encoding="ascii", newline="") as handle:
            reader = csv.DictReader(handle, delimiter="\t")
            if tuple(reader.fieldnames or ()) != FIELDS:
                raise ValueError("candidate inventory fields are invalid")
            rows = list(reader)
    except (OSError, UnicodeError, csv.Error) as error:
        raise ValueError("candidate inventory could not be read") from error
    if not 5 <= len(rows) <= 10:
        raise ValueError("candidate inventory must contain 5 through 10 rows")
    if len({row["id"] for row in rows}) != len(rows):
        raise ValueError("candidate inventory contains duplicate identifiers")
    if tuple(row["path"] for row in rows) != CANDIDATE_PATHS:
        raise ValueError("candidate inventory differs from the reviewed surface")
    for row in rows:
        if (
            row["method"] != "GET"
            or row["pagination"] not in ("cursor", "none")
            or row["stability"] != "production"
            or not row["response_type"]
        ):
            raise ValueError("candidate inventory is not stable read-only")
    return rows


def check_evidence(root: Path = ROOT) -> None:
    plugin = validate_plugin(
        read_bounded_json(
            root / "provider-drift/plugins/normalized-json-v1.json", "plugin"
        )
    )
    lock = validate_lock(
        read_bounded_json(
            root / "provider-drift/providers/ovhcloud-v2-probe.lock.json",
            "OVHcloud probe lock",
        )
    )
    observation = validate_observation(
        read_bounded_json(
            root / "provider-drift/providers/ovhcloud-v2-probe.observed.json",
            "OVHcloud probe observation",
        )
    )
    expected_plugin = {"id": plugin["id"], "version": plugin["version"]}
    if lock["plugin"] != expected_plugin or observation["plugin"] != expected_plugin:
        raise ValueError("OVHcloud probe plugin identity is inconsistent")
    if build_report(lock, observation)["result"] != "clean":
        raise ValueError("OVHcloud probe lock and observation have drifted")

    inventory = read_inventory(root / "provider-probes/ovhcloud-v2/CANDIDATES.tsv")
    rows = lock["contracts"]["operations"]
    if len(rows) != len(inventory):
        raise ValueError("OVHcloud lock candidate count is inconsistent")
    for expected, actual in zip(inventory, rows):
        values = actual["values"]
        pagination = (
            "cursor"
            if values["headers"]
            == ["X-Pagination-Cursor", "X-Pagination-Size"]
            else "none"
        )
        projected = {
            "id": actual["id"],
            "method": values["method"],
            "path": values["path"],
            "pagination": pagination,
            "response_type": values["response_type"],
            "stability": values["stability"],
        }
        if projected != expected:
            raise ValueError("OVHcloud inventory and source lock differ")


def ensure_no_probe_packages(
    package_names: set[str], planned_names: set[str], publish_order: tuple[str, ...]
) -> None:
    for source, names in (
        ("Cargo package metadata", package_names),
        ("release plan", planned_names),
        ("publisher", set(publish_order)),
    ):
        if any("ovhcloud" in name.lower() for name in names):
            raise ValueError(f"OVHcloud probe entered the {source}")


def check_no_publish(root: Path = ROOT) -> None:
    if any((root / "provider-probes").rglob("Cargo.toml")):
        raise ValueError("provider probes must not be Cargo packages")
    metadata = json.loads(release_crates.capture(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"]
    ))
    package_names = {package["name"] for package in metadata["packages"]}
    plan = release_crates.release_plan(root / "release-crates.toml")
    ensure_no_probe_packages(
        package_names, set(plan["crates"]), release_crates.PUBLISH_ORDER
    )


def main() -> int:
    try:
        check_evidence()
        check_no_publish()
    except (OSError, RuntimeError, UnicodeError, ValueError) as error:
        print(f"OVHcloud probe: {error}", file=sys.stderr)
        return 1
    print("OVHcloud v2 probe is source-locked, read-only, and unpublished.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
