#!/usr/bin/env python3
"""Regression tests for the Hetzner neutral-drift bridge."""

from __future__ import annotations

from pathlib import Path
import shutil
import subprocess
import tempfile

import check_hetzner_provider_drift_bridge as bridge
from provider_drift_model import read_bounded_json, validate_lock
from provider_drift_model import validate_observation


ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "provider-drift" / "providers" / "hetzner.lock.json"
OBSERVATION = ROOT / "provider-drift" / "providers" / "hetzner.observed.json"


def fixtures() -> tuple[dict, dict]:
    return (
        validate_lock(read_bounded_json(LOCK, "provider lock")),
        validate_observation(read_bounded_json(OBSERVATION, "provider observation")),
    )


def assert_raises(expected: str, function, *args) -> None:
    try:
        function(*args)
    except bridge.BridgeError as error:
        assert expected in str(error), error
        return
    raise AssertionError("expected BridgeError")


def test_tracked_bridge_matches_authoritative_locks() -> None:
    lock, observation = fixtures()
    bridge.validate_bridge(lock, observation)


def test_lock_and_observation_must_change_together() -> None:
    lock, observation = fixtures()
    observation["contracts"]["schemas"] = []
    assert_raises("observation differs", bridge.validate_bridge, lock, observation)


def test_joint_digest_tampering_is_detected_against_repository() -> None:
    lock, observation = fixtures()
    for document in (lock, observation):
        document["contracts"]["schemas"][0]["values"]["sha256"] = "0" * 64
    assert_raises("digest is stale", bridge.validate_bridge, lock, observation)


def test_joint_source_rotation_must_match_authoritative_checker() -> None:
    lock, observation = fixtures()
    for document in (lock, observation):
        document["sources"][0]["sha256"] = "f" * 64
    assert_raises("source pin is stale", bridge.validate_bridge, lock, observation)


def test_evidence_path_escape_is_rejected() -> None:
    lock, observation = fixtures()
    for document in (lock, observation):
        document["contracts"]["schemas"][0]["values"]["path"] = "../outside"
    assert_raises("escapes the repository", bridge.validate_bridge, lock, observation)


def test_count_tampering_is_detected() -> None:
    lock, observation = fixtures()
    for document in (lock, observation):
        document["contracts"]["operations"][0]["values"]["count"] = 220
    assert_raises("row count is stale", bridge.validate_bridge, lock, observation)


def test_provider_and_plugin_identity_are_fixed() -> None:
    lock, observation = fixtures()
    lock["provider"] = "other"
    observation["provider"] = "other"
    assert_raises("provider must remain Hetzner", bridge.validate_bridge, lock, observation)
    lock, observation = fixtures()
    lock["plugin"]["version"] = 2
    observation["plugin"]["version"] = 2
    assert_raises("plugin identity", bridge.validate_bridge, lock, observation)


def test_nested_evidence_descriptors_are_typed_before_file_access() -> None:
    lock, observation = fixtures()
    for document in (lock, observation):
        document["contracts"]["schemas"][0]["values"]["path"] = 42
    assert_raises("descriptor is invalid", bridge.validate_bridge, lock, observation)


def test_local_policy_evidence_tampering_is_detected() -> None:
    lock, observation = fixtures()
    for document in (lock, observation):
        headers = {
            row["id"]: row["values"] for row in document["contracts"]["headers"]
        }
        headers["rate-limit-policy"]["evidence"]["sha256"] = "0" * 64
    assert_raises("digest is stale", bridge.validate_bridge, lock, observation)


def main() -> None:
    tests = tuple(
        value
        for name, value in globals().items()
        if name.startswith("test_") and callable(value)
    )
    for test in tests:
        test()
    print(f"{len(tests)} Hetzner provider drift bridge tests passed.")


def test_surface_gate_checks_bridge_and_propagates_its_failure() -> None:
    names = ["check_hetzner_api_drift.py", "check_robot_api_lock.py",
             "check_hetzner_changelog.py", "check_hetzner_provider_drift_bridge.py"]
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        scripts = root / "scripts"
        scripts.mkdir()
        shutil.copyfile(ROOT / "scripts/check_hetzner_api_surface.sh", scripts / "surface.sh")
        for mode in ("--local-only", "--fetch"):
            for status in (0, 37):
                for name in names:
                    path = scripts / name
                    code = status if name == names[-1] else 0
                    path.write_text(f"#!/bin/sh\nprintf '%s\\n' '{name}' >> calls\nexit {code}\n",
                                    encoding="ascii")
                    path.chmod(0o700)
                log = root / "calls"
                log.unlink(missing_ok=True)
                result = subprocess.run(["sh", "scripts/surface.sh", mode], cwd=root,
                                        capture_output=True, text=True, check=False)
                assert result.returncode == status, result
                assert log.read_text(encoding="ascii").splitlines() == names
                assert ("All tracked" in result.stdout) == (status == 0)


if __name__ == "__main__":
    main()
