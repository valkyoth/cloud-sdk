#!/usr/bin/env python3
"""Regression tests for the v0.85 Robot boot source-lock checker."""

from __future__ import annotations

from copy import deepcopy
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/check_robot_boot.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("check_robot_boot", SCRIPT)
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
    for field in ["source", "identity_fields", "families", "quota", "policy"]:
        changed = deepcopy(value)
        if field == "identity_fields":
            changed[field].reverse()
        elif field == "families":
            changed[field]["windows"]["activation_fields"].reverse()
        else:
            first = next(iter(changed[field]))
            changed[field][first] = "changed"
        mutations.append((field, changed))
    for field, replacement in [
        ("method", "PATCH"),
        ("path", "/boot/{server-ip}"),
        ("family", "future"),
        ("state", "unknown"),
        ("input", ["arch"]),
        ("errors", []),
    ]:
        changed = deepcopy(value)
        changed["operations"][0][field] = replacement
        mutations.append((f"operation {field}", changed))
    missing = deepcopy(value)
    missing["operations"].pop()
    mutations.append(("missing operation", missing))
    for name, changed in mutations:
        try:
            checker.validate_contract(changed)
        except SystemExit:
            pass
        else:
            raise AssertionError(f"changed {name} was accepted")
    print(f"{len(mutations)} Robot boot contract mutation groups passed.")


if __name__ == "__main__":
    main()
