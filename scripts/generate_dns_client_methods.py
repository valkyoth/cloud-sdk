#!/usr/bin/env python3
"""Generate exhaustive service-typed Hetzner DNS client methods."""

from __future__ import annotations

import argparse

import generate_cloud_client_methods as shared

OUTPUT = shared.ROOT / "crates/cloud-sdk-hetzner/src/client/dns.rs"
EXPECTED_DNS_OPERATIONS = 24


def render() -> str:
    return shared.render_service(
        service="dns",
        service_label="DNS",
        type_prefix="Dns",
        service_marker="DnsService",
        expected_operations=EXPECTED_DNS_OPERATIONS,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    generated = render()
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text(encoding="ascii") != generated:
            raise SystemExit("DNS client methods are stale; regenerate them")
        print(f"{EXPECTED_DNS_OPERATIONS} DNS client operations are current.")
        return 0
    OUTPUT.write_text(generated, encoding="ascii")
    print(f"generated {EXPECTED_DNS_OPERATIONS} DNS client operations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
