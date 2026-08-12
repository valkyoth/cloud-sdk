#!/usr/bin/env python3
"""Regression tests for the v0.82 Robot reset source-lock checker."""

from __future__ import annotations

from copy import deepcopy
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/check_robot_resets.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("check_robot_resets", SCRIPT)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> None:
    checker = load_checker()
    value, payload = checker.read_lock()
    checker.validate_contract(value)
    assert len(payload) <= checker.MAX_LOCK_BYTES

    mutations = []
    missing = deepcopy(value)
    missing["operations"].pop()
    mutations.append(("missing operation", missing))
    for field, replacement in [
        ("method", "DELETE"),
        ("path", "/changed"),
        ("request_fields", ["unsafe"]),
        ("success", {"status": 201, "body": "json", "shape": "reset-action"}),
        ("errors", [{"status": 404, "code": "WRONG"}]),
        ("quota", {"requests": 501, "seconds": 3600}),
    ]:
        changed = deepcopy(value)
        changed["operations"][0][field] = replacement
        mutations.append((f"operation {field}", changed))
    for field in ["source", "detail_fields", "reset_types", "source_inconsistencies", "policy"]:
        changed = deepcopy(value)
        if isinstance(changed[field], list):
            changed[field].reverse()
        else:
            first = next(iter(changed[field]))
            changed[field][first] = "changed"
        mutations.append((field, changed))

    for name, changed in mutations:
        try:
            checker.validate_contract(changed)
        except SystemExit:
            pass
        else:
            raise AssertionError(f"changed {name} was accepted")
    print(f"{len(mutations)} Robot reset contract mutation groups passed.")


if __name__ == "__main__":
    main()
