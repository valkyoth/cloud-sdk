#!/usr/bin/env python3
"""Regression tests for direct third-party dependency pinning."""

from __future__ import annotations

import json
import tempfile
from pathlib import Path

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
    dependencies = {
        "hyper": {"version": "=1.11.1"},
        "sanitization": {"version": "=2.0.4", "default-features": False},
        "cloud-sdk": {"version": "1.1.0", "path": "crates/cloud-sdk"},
    }
    assert pins.direct_pins(dependencies) == {
        "hyper": "1.11.1",
        "sanitization": "2.0.4",
    }
    current = {"hyper": "1.11.1", "sanitization": "2.0.4"}
    assert pins.freshness_problems(dependencies, current.__getitem__) == []
    stale = {"hyper": "1.11.1", "sanitization": "2.0.5"}
    assert pins.freshness_problems(dependencies, stale.__getitem__) == [
        "sanitization: crates.io reports 2.0.5; exact reviewed pin is 2.0.4"
    ]
    payload = json.dumps(
        {
            "crate": {
                "name": "rustls",
                "max_version": "0.24.0-dev.1",
                "max_stable_version": "0.23.43",
            }
        }
    ).encode("ascii")
    assert pins.parse_registry_payload(payload, "rustls") == "0.23.43"
    try:
        pins.parse_registry_payload(payload, "hyper")
    except ValueError as error:
        assert "mismatched metadata" in str(error)
    else:
        raise AssertionError("mismatched crates.io identity was accepted")
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        (root / "Cargo.toml").write_text('[workspace.dependencies]\nsyn="=3.0.6"\n')
        for name in pins.AUXILIARY_MANIFESTS:
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('[dependencies]\nsyn="=3.0.4"\n[target.test.build-dependencies]\nserde="=1.0.228"\n')
        tables = pins.dependency_tables(root)
        assert len(tables) == 7
        current = {"syn": "3.0.6", "serde": "1.0.229"}
        stale = [(label, problem) for label, deps in tables
                 for problem in pins.freshness_problems(deps, current.__getitem__)]
        assert len(stale) == 6
        assert all(label != "workspace" for label, _ in stale)
        (root / pins.AUXILIARY_MANIFESTS[0]).unlink()
        try:
            pins.dependency_tables(root)
        except FileNotFoundError:
            pass
        else:
            raise AssertionError("missing auxiliary workspace silently skipped")
    print("11 exact dependency pin and freshness regression groups passed.")


if __name__ == "__main__":
    main()
