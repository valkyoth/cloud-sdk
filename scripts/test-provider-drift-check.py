#!/usr/bin/env python3
"""Data-flow tests for live provider drift observations."""

from __future__ import annotations

import copy
from pathlib import Path

import check_provider_drift as checker
from provider_drift_adapters import AdapterError, build_live_observation
from provider_drift_model import read_bounded_json, validate_lock, validate_observation
from provider_drift_model import validate_plugin


ROOT = Path(__file__).resolve().parents[1]
PLUGIN = ROOT / "provider-drift" / "plugins" / "normalized-json-v1.json"
LOCK = ROOT / "provider-drift" / "providers" / "hetzner.lock.json"
OBSERVATION = ROOT / "provider-drift" / "providers" / "hetzner.observed.json"


def fixtures() -> tuple[dict, dict, dict]:
    return (
        validate_plugin(read_bounded_json(PLUGIN, "plugin")),
        validate_lock(read_bounded_json(LOCK, "provider lock")),
        validate_observation(read_bounded_json(OBSERVATION, "provider observation")),
    )


def test_live_adapter_output_is_used_and_must_match_tracked_evidence() -> None:
    plugin, lock, tracked = fixtures()
    original = checker.build_live_observation
    called = []

    def live(_lock: dict, payloads: dict) -> dict:
        called.append(payloads)
        return copy.deepcopy(tracked)

    checker.build_live_observation = live
    try:
        report = checker.evaluate(plugin, lock, tracked, {"authenticated": b"bytes"})
        assert report["result"] == "clean"
        stale = copy.deepcopy(tracked)
        stale["contracts"]["schemas"] = []
        try:
            checker.evaluate(plugin, lock, stale, {"authenticated": b"bytes"})
        except checker.ModelError as error:
            assert "differs from tracked evidence" in str(error)
        else:
            raise AssertionError("expected stale tracked evidence to fail")
    finally:
        checker.build_live_observation = original
    assert called == [{"authenticated": b"bytes"}, {"authenticated": b"bytes"}]


def test_manifest_identity_cannot_select_an_unreviewed_adapter() -> None:
    _plugin, lock, _tracked = fixtures()
    lock["provider"] = "unreviewed"
    try:
        build_live_observation(lock, {})
    except AdapterError as error:
        assert "no reviewed source adapter" in str(error)
    else:
        raise AssertionError("expected AdapterError")


def main() -> None:
    tests = tuple(
        value
        for name, value in globals().items()
        if name.startswith("test_") and callable(value)
    )
    for test in tests:
        test()
    print(f"{len(tests)} provider drift data-flow tests passed.")


if __name__ == "__main__":
    main()
