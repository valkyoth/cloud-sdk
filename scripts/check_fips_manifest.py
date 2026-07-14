#!/usr/bin/env python3
"""Require exact published constraints for the reviewed FIPS dependency set."""

from __future__ import annotations

import json
import subprocess


EXPECTED = {
    "aws-lc-fips-sys": "=0.13.15",
    "aws-lc-rs": "=1.17.1",
    "aws-lc-sys": "=0.42.0",
    "reqwest": "=0.13.4",
    "rustls": "=0.23.42",
    "rustls-platform-verifier": "=0.7.0",
}

metadata = json.loads(
    subprocess.check_output(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"],
        text=True,
    )
)
package = next(
    item for item in metadata["packages"] if item["name"] == "cloud-sdk-reqwest"
)
requirements = {item["name"]: item["req"] for item in package["dependencies"]}
for name, expected in EXPECTED.items():
    actual = requirements.get(name)
    if actual != expected:
        raise SystemExit(
            f"FIPS manifest constraint for {name} must be {expected}, found {actual}"
        )

print("6 exact FIPS manifest constraints passed.")
