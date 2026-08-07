#!/usr/bin/env python3
"""Regression tests for OVHcloud cursor and schema conformance evidence."""

from __future__ import annotations

import copy
import runpy

from check_ovhcloud_header_conformance import ROOT, check, fixture_row
from ovhcloud_probe_adapter import OvhcloudProbeError, build_observation


PROBE_TESTS = runpy.run_path(str(ROOT / "scripts/test-ovhcloud-probe.py"))
lock = PROBE_TESTS["lock"]
payloads = PROBE_TESTS["payloads"]


def test_current_evidence_is_consistent() -> None:
    check()
    row = fixture_row()
    assert row["terminal"] == "next_header_absent"
    assert row["schema_version"] == "1.0"


def test_adapter_binds_schema_example_to_console_major() -> None:
    source = payloads()
    observed = build_observation(lock(), source)
    schema = observed["contracts"]["headers"][1]["values"]
    assert schema["reviewed_version"] == "1.0"
    assert schema["reviewed_major"] == 1

    changed = copy.deepcopy(source)
    changed["api-v2-principles"] = changed["api-v2-principles"].replace(
        b"X-Schemas-Version: 1.0", b"X-Schemas-Version: 2.0"
    )
    try:
        build_observation(lock(), changed)
    except OvhcloudProbeError:
        pass
    else:
        raise AssertionError("schema major drift was accepted")


def main() -> None:
    tests = [
        test_current_evidence_is_consistent,
        test_adapter_binds_schema_example_to_console_major,
    ]
    for test in tests:
        test()
    print(f"{len(tests)} OVHcloud header conformance tests passed.")


if __name__ == "__main__":
    main()
