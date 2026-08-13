#!/usr/bin/env python3
"""Regression tests for the v0.86 Robot reverse-DNS source-lock checker."""

from __future__ import annotations

from copy import deepcopy
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/check_robot_rdns.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("check_robot_rdns", SCRIPT)
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
    for field in ["source", "quota", "policy"]:
        changed = deepcopy(value)
        first = next(iter(changed[field]))
        changed[field][first] = "changed"
        mutations.append((field, changed))
    for field, replacement in [
        ("method", "PATCH"),
        ("path", "/rdns/{unvalidated}"),
        ("success", [200]),
        ("input", ["future"]),
        ("output", "unknown"),
        ("errors", []),
    ]:
        changed = deepcopy(value)
        changed["operations"][2][field] = replacement
        mutations.append((f"operation {field}", changed))
    missing = deepcopy(value)
    missing["operations"].pop()
    mutations.append(("missing operation", missing))
    duplicate = deepcopy(value)
    duplicate["operations"][1]["id"] = duplicate["operations"][0]["id"]
    mutations.append(("duplicate operation", duplicate))
    for name, changed in mutations:
        try:
            checker.validate_contract(changed)
        except SystemExit:
            pass
        else:
            raise AssertionError(f"changed {name} was accepted")
    print(f"{len(mutations)} Robot reverse-DNS contract mutation groups passed.")


if __name__ == "__main__":
    main()
