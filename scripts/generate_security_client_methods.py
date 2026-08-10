#!/usr/bin/env python3
"""Generate exhaustive service-typed Hetzner security client methods."""

from __future__ import annotations

import argparse

import generate_cloud_client_methods as shared

OUTPUT = shared.ROOT / "crates/cloud-sdk-hetzner/src/client/security.rs"
EXPECTED_SECURITY_OPERATIONS = 14


def render() -> str:
    return shared.render_service(
        service="security",
        service_label="Security",
        type_prefix="Security",
        service_marker="SecurityService",
        expected_operations=EXPECTED_SECURITY_OPERATIONS,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    generated = render()
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text(encoding="ascii") != generated:
            raise SystemExit("Security client methods are stale; regenerate them")
        print(f"{EXPECTED_SECURITY_OPERATIONS} Security client operations are current.")
        return 0
    OUTPUT.write_text(generated, encoding="ascii")
    print(f"generated {EXPECTED_SECURITY_OPERATIONS} Security client operations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
