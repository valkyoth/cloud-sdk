#!/usr/bin/env python3
"""Generate exhaustive service-typed Hetzner Storage client methods."""

from __future__ import annotations

import argparse

import generate_cloud_client_methods as shared

OUTPUT = shared.ROOT / "crates/cloud-sdk-hetzner/src/client/storage.rs"
EXPECTED_STORAGE_OPERATIONS = 31


def render() -> str:
    return shared.render_service(
        service="storage",
        service_label="Storage",
        type_prefix="Storage",
        service_marker="StorageService",
        expected_operations=EXPECTED_STORAGE_OPERATIONS,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    generated = render()
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text(encoding="ascii") != generated:
            raise SystemExit("Storage client methods are stale; regenerate them")
        print(f"{EXPECTED_STORAGE_OPERATIONS} Storage client operations are current.")
        return 0
    OUTPUT.write_text(generated, encoding="ascii")
    print(f"generated {EXPECTED_STORAGE_OPERATIONS} Storage client operations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
