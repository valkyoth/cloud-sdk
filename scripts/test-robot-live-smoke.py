#!/usr/bin/env python3
"""Regression tests for the Robot live-smoke source contract."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parent.parent
CHECK = ROOT / "scripts/check_robot_live_smoke.py"


def load_checker():
    specification = importlib.util.spec_from_file_location("robot_live_check", CHECK)
    assert specification is not None and specification.loader is not None
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def main() -> None:
    checker = load_checker()
    checker.checked_text("fixture", "required", ("required",), ("forbidden",))
    for text, required, forbidden in (
        ("", ("required",), ()),
        ("forbidden", (), ("forbidden",)),
    ):
        try:
            checker.checked_text("fixture", text, required, forbidden)
        except ValueError:
            pass
        else:
            raise AssertionError("invalid source contract was accepted")

    result = subprocess.run(
        [str(CHECK)], cwd=ROOT, check=False, capture_output=True, text=True
    )
    assert result.returncode == 0, result
    assert result.stdout.strip() == "Robot live smoke source contract passed."
    print("3 Robot live smoke contract tests passed.")


if __name__ == "__main__":
    main()
