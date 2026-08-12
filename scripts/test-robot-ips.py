#!/usr/bin/env python3
"""Regression tests for the v0.80 Robot IP contract checker."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/check_robot_ips.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("check_robot_ips", SCRIPT)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> None:
    checker = load_checker()
    value, payload = checker.read_lock()
    checker.validate_contract(value)
    assert len(payload) <= checker.MAX_LOCK_BYTES

    changed = dict(value)
    changed["operations"] = list(value["operations"][:-1])
    try:
        checker.validate_contract(changed)
    except SystemExit:
        pass
    else:
        raise AssertionError("missing Robot IP operation was accepted")

    changed = dict(value)
    changed["policy"] = dict(value["policy"])
    changed["policy"]["delete_mac_permit"] = "mutation"
    try:
        checker.validate_contract(changed)
    except SystemExit:
        pass
    else:
        raise AssertionError("weakened destructive policy was accepted")

    print("2 Robot IP contract regression groups passed.")


if __name__ == "__main__":
    main()
