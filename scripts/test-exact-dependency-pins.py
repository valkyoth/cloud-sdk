#!/usr/bin/env python3
"""Regression tests for direct third-party dependency pinning."""

from __future__ import annotations

import check_exact_dependency_pins as pins


def main() -> None:
    assert pins.pin_problems(
        {
            "hyper": {"version": "=1.11.0"},
            "serde": "=1.0.229",
            "cloud-sdk": {"version": "1.0.0", "path": "crates/cloud-sdk"},
        }
    ) == []
    assert pins.pin_problems({"hyper": {"version": "1.11.0"}}) == [
        "hyper: direct third-party requirement must be an exact =X.Y.Z pin"
    ]
    assert pins.pin_problems({"hyper": {"version": ">=1.11.0"}})
    assert pins.pin_problems({"hyper": {"version": "=1.11"}})
    assert pins.pin_problems({"hyper": {"git": "https://example.invalid/repo"}})
    print("5 exact dependency pin regression groups passed.")


if __name__ == "__main__":
    main()
