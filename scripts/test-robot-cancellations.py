#!/usr/bin/env python3
"""Regression tests for the v0.79 Robot cancellation checker."""

from __future__ import annotations

import copy
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/check_robot_cancellations.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("check_robot_cancellations", SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load cancellation checker")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


checker = load_checker()


def assert_exits(expected: str, function, *args) -> None:
    try:
        function(*args)
    except SystemExit as error:
        if expected not in str(error):
            raise AssertionError(f"expected {expected!r} in {error!r}") from error
        return
    raise AssertionError("expected SystemExit")


def contract():
    return checker.read_lock()[0]


def operation(value, operation_id: str):
    return next(item for item in value["operations"] if item["id"] == operation_id)


def test_committed_contract_and_implementation_pass() -> None:
    checker.validate_contract(contract())
    checker.validate_implementation()


def test_route_and_method_drift_fail_closed() -> None:
    value = copy.deepcopy(contract())
    operation(value, "robot_delete_subnet_cancellation")["path"] = "/ip/{ip}/cancellation"
    assert_exits("operation route policy changed", checker.validate_contract, value)
    value = copy.deepcopy(contract())
    operation(value, "robot_create_server_cancellation")["method"] = "PUT"
    assert_exits("operation route policy changed", checker.validate_contract, value)


def test_target_specific_delete_body_and_safety_drift_fail_closed() -> None:
    value = copy.deepcopy(contract())
    operation(value, "robot_delete_server_cancellation")["success"] = {
        "status": 200, "body": "json", "envelope": "cancellation",
    }
    assert_exits("server DELETE success", checker.validate_contract, value)
    value = copy.deepcopy(contract())
    operation(value, "robot_delete_ip_cancellation")["success"] = {
        "status": 200, "body": "empty", "envelope": None,
    }
    assert_exits("JSON success envelope", checker.validate_contract, value)
    value = copy.deepcopy(contract())
    value["policy"]["create_retry"] = "automatic"
    assert_exits("safety policy changed", checker.validate_contract, value)


def test_source_variants_and_identity_are_locked() -> None:
    value = copy.deepcopy(contract())
    value["ip_response"]["date_field_variants"] = ["cancellation_date"]
    assert_exits("date variants changed", checker.validate_contract, value)
    value = copy.deepcopy(contract())
    value["source"]["sha256"] = "0" * 64
    assert_exits("source identity changed", checker.validate_contract, value)


def main() -> None:
    tests = [value for name, value in globals().items() if name.startswith("test_")]
    for test in tests:
        test()
    print(f"{len(tests)} Robot cancellation contract regression tests passed.")


if __name__ == "__main__":
    main()
