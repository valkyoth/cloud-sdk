#!/usr/bin/env python3
"""Regression tests for the v0.78 Robot server contract checker."""

from __future__ import annotations

import copy
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/check_robot_server_contract.py"
SPEC = importlib.util.spec_from_file_location("robot_server_contract", SCRIPT)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("could not load Robot server checker")
checker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checker)


def assert_exits(message: str, function, *args) -> None:
    try:
        function(*args)
    except SystemExit as error:
        if message not in str(error):
            raise AssertionError(f"expected {message!r} in {error!r}") from error
        return
    raise AssertionError("expected SystemExit")


def main() -> None:
    value = checker.read_json(checker.FIXTURE)
    checker.validate_fixture(value)
    checker.validate_api_relationship(value, checker.read_json(checker.API_LOCK, 256 * 1024))

    changed = copy.deepcopy(value)
    changed["status_values"].append("future")
    assert_exits("status values changed", checker.validate_fixture, changed)

    changed = copy.deepcopy(value)
    changed["deprecated_aliases"].clear()
    assert_exits("deprecated alias policy changed", checker.validate_fixture, changed)

    api = checker.read_json(checker.API_LOCK, 256 * 1024)
    changed_api = copy.deepcopy(api)
    next(row for row in changed_api["operations"] if row["id"] == "get_server")["path"] = "/server/{server-ip}"
    assert_exits("server operations differ", checker.validate_api_relationship, value, changed_api)
    checker.validate_implementation()
    print("3 Robot server contract regression groups passed.")


if __name__ == "__main__":
    main()
