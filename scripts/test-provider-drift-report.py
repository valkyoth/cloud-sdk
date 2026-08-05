#!/usr/bin/env python3
"""Determinism and classification tests for provider drift reports."""

from __future__ import annotations

import copy
from pathlib import Path

from provider_drift_model import canonical_bytes, read_bounded_json, validate_lock
from provider_drift_model import validate_observation
from provider_drift_report import build_report


ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "provider-drift" / "providers" / "hetzner.lock.json"
OBSERVATION = ROOT / "provider-drift" / "providers" / "hetzner.observed.json"


def fixtures() -> tuple[dict, dict]:
    lock = validate_lock(read_bounded_json(LOCK, "provider lock"))
    observation = validate_observation(
        read_bounded_json(OBSERVATION, "provider observation")
    )
    return lock, observation


def test_tracked_hetzner_bridge_is_clean() -> None:
    lock, observation = fixtures()
    report = build_report(lock, observation)
    assert report == {
        "changes": [],
        "format": "cloud-sdk-provider-drift-report/v1",
        "provider": "hetzner",
        "result": "clean",
    }


def test_shuffled_rows_and_fields_render_identically() -> None:
    lock, observation = fixtures()
    baseline = canonical_bytes(build_report(lock, observation))
    shuffled = copy.deepcopy(observation)
    shuffled["sources"].reverse()
    for rows in shuffled["contracts"].values():
        rows.reverse()
    assert canonical_bytes(build_report(lock, shuffled)) == baseline


def test_changes_are_field_level_owned_and_payload_free() -> None:
    lock, observation = fixtures()
    hostile = "tenant-secret-do-not-print"
    observation["contracts"]["operations"][0]["values"]["path"] = hostile
    report = build_report(lock, observation)
    assert report["result"] == "drift"
    assert len(report["changes"]) == 1
    change = report["changes"][0]
    assert change["category"] == "operations"
    assert change["fields"] == ["path"]
    assert change["owner"] == "hetzner-maintainers"
    assert change["severity"] == "blocking"
    assert hostile.encode("ascii") not in canonical_bytes(report)


def test_source_digest_rotation_is_security_owned_and_blocking() -> None:
    lock, observation = fixtures()
    observation["sources"][0]["sha256"] = "f" * 64
    change = build_report(lock, observation)["changes"][0]
    assert change["category"] == "sources"
    assert change["fields"] == ["sha256"]
    assert change["owner"] == "security-maintainers"
    assert change["severity"] == "blocking"


def test_additions_and_removals_follow_explicit_compatibility_policy() -> None:
    lock, observation = fixtures()
    observation["contracts"]["schemas"].append(
        {"id": "new-schema", "values": {"type": "object"}}
    )
    observation["contracts"]["pagination"] = []
    changes = build_report(lock, observation)["changes"]
    added = next(change for change in changes if change["kind"] == "added")
    removed = next(change for change in changes if change["kind"] == "removed")
    assert added["category"] == "schemas" and added["severity"] == "review"
    assert removed["category"] == "pagination"
    assert removed["severity"] == "blocking"


def test_plugin_or_provider_substitution_is_blocking() -> None:
    lock, observation = fixtures()
    observation["plugin"]["version"] = 2
    observation["provider"] = "attacker"
    changes = build_report(lock, observation)["changes"]
    assert [change["id"] for change in changes] == ["plugin", "provider"]
    assert all(change["severity"] == "blocking" for change in changes)


def main() -> None:
    tests = tuple(
        value
        for name, value in globals().items()
        if name.startswith("test_") and callable(value)
    )
    for test in tests:
        test()
    print(f"{len(tests)} provider drift report tests passed.")


if __name__ == "__main__":
    main()
