#!/usr/bin/env python3
"""Regression tests for OVHcloud task conformance evidence."""

from __future__ import annotations

import copy
import tempfile
from pathlib import Path

from check_ovhcloud_task_conformance import (
    FIELDS,
    ROOT,
    check,
    expected_row,
    fixture_row,
)
from provider_drift_model import read_bounded_json


def assert_rejected(action) -> None:
    try:
        action()
    except ValueError:
        return
    raise AssertionError("invalid OVHcloud task evidence was accepted")


def current_lock() -> dict:
    return read_bounded_json(
        ROOT / "provider-drift/providers/ovhcloud-v2-probe.lock.json", "lock"
    )


def require_fixture_match(lock: dict) -> None:
    if expected_row(lock) != fixture_row():
        raise ValueError("task contract no longer matches the fixture")


def test_current_evidence_is_exact_and_event_is_fixture_only() -> None:
    check()
    row = fixture_row()
    assert row["resource_response"] == "common.Task"
    assert row["generic_event_scope"] == "fixture-only"
    assert row["generic_event_path"] == "/event"


def test_route_method_status_and_model_drift_fail_closed() -> None:
    for category, row_id, field, replacement in (
        (
            "operations",
            "notification/contactmean/by-contactmeanid/task",
            "method",
            "POST",
        ),
        (
            "operations",
            "notification/contactmean/by-contactmeanid/task/by-taskid",
            "stability",
            "beta",
        ),
        ("schemas", "notification-task-models", "statuses", ["DONE"]),
    ):
        changed = copy.deepcopy(current_lock())
        rows = changed["contracts"][category]
        one = next(row for row in rows if row["id"] == row_id)
        one["values"][field] = replacement
        assert_rejected(lambda value=changed: require_fixture_match(value))


def test_fixture_parser_rejects_shape_count_and_encoding() -> None:
    header = "\t".join(FIELDS)
    row = "\t".join(fixture_row()[field] for field in FIELDS)
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "tasks.tsv"
        path.write_text(f"{header}\n{row}\n", encoding="ascii")
        assert fixture_row(path)["resource_response"] == "common.Task"
        path.write_text(f"{header}\n{row}\n{row}\n", encoding="ascii")
        assert_rejected(lambda: fixture_row(path))
        path.write_text("wrong\n", encoding="ascii")
        assert_rejected(lambda: fixture_row(path))
        path.write_bytes(header.encode("ascii") + b"\n\xff\n")
        assert_rejected(lambda: fixture_row(path))


def main() -> None:
    tests = (
        test_current_evidence_is_exact_and_event_is_fixture_only,
        test_route_method_status_and_model_drift_fail_closed,
        test_fixture_parser_rejects_shape_count_and_encoding,
    )
    for test in tests:
        test()
    print(f"{len(tests)} OVHcloud task conformance regression groups passed.")


if __name__ == "__main__":
    main()
