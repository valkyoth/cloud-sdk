#!/usr/bin/env python3
"""Validate the complete reviewed Cargo fuzz binary inventory."""

from pathlib import Path
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
TARGETS = (
    "buffer_writers",
    "request_targets",
    "action_requests",
    "labels_dns",
    "pagination",
    "quota_retry",
    "retry_policy",
    "pagination_opaque",
    "provider_links",
    "action_polling",
    "response_envelopes",
    "response_content_type",
    "checked_response",
    "cloud_special_responses",
    "raw_response_parser",
    "raw_http1_wire",
    "incremental_json",
    "robot_form",
    "robot_error_protocol",
    "robot_server_response",
    "robot_ip_parser",
    "robot_cancellation_response",
    "robot_ip_response",
    "robot_subnet_response",
    "robot_reset_response",
    "robot_failover_response",
    "robot_boot_response",
    "robot_rdns_response",
    "robot_traffic_response",
    "robot_ssh_key_response",
    "robot_firewall_response",
    "robot_vswitch_response",
    "robot_ordering_response",
    "robot_transaction_response",
    "metadata_response",
    "cratesio_targets",
    "cratesio_metadata",
    "cratesio_continuation",
    "cratesio_redirect",
)


def validate(manifest):
    if manifest.get("package", {}).get("autobins") is not False:
        raise ValueError("automatic fuzz binary discovery must be disabled")
    expected = [
        {"name": name, "path": f"fuzz_targets/{name}.rs",
         "test": False, "doc": False, "bench": False}
        for name in TARGETS
    ]
    if manifest.get("bin") != expected:
        raise ValueError("fuzz target name/path inventory differs from review")


def main():
    if len(sys.argv) > 2:
        raise ValueError("usage: check_fuzz_inventory.py [manifest]")
    path = Path(sys.argv[1]) if len(sys.argv) == 2 else ROOT / "fuzz/Cargo.toml"
    with path.open("rb") as source:
        validate(tomllib.load(source))
    print(" ".join(TARGETS))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, TypeError, AttributeError):
        print("fuzz inventory: invalid manifest or unreviewed binary inventory", file=sys.stderr)
        raise SystemExit(1) from None
