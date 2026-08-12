#!/usr/bin/env python3
"""Validate the v0.83 Robot failover source lock."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "tests/fixtures/robot-failover/v0.83.0.json"
LOCK_SHA256 = "a6f4a41daf9bf50dc5601d80260eded89bca1b508069f9023dbb8a1fb7d75854"
MAX_LOCK_BYTES = 16 * 1024
SOURCE_SHA256 = "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a"
PREPARE_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/failover/prepare.rs"
DECODE_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/failover/decode.rs"
EXCHANGE_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/failover/exchange.rs"
FUZZ_SOURCE = ROOT / "fuzz/fuzz_targets/robot_failover_response.rs"
FUZZ_HARNESS = ROOT / "scripts/check_fuzz_harness.sh"
FUZZ_SEEDS = ROOT / "fuzz/seeds/robot_failover_response"


def operation(
    method: str,
    path: str,
    fields: list[str],
    shape: str,
    errors: list[tuple[int, str]],
    requests: int,
) -> dict[str, Any]:
    return {
        "method": method,
        "path": path,
        "request_fields": fields,
        "success": {"status": 200, "body": "json", "shape": shape},
        "errors": [{"status": status, "code": code} for status, code in errors],
        "quota": {"requests": requests, "seconds": 3600},
    }


EXPECTED_OPERATIONS = {
    "robot_list_failovers": operation(
        "GET", "/failover", [], "failover-list", [(404, "NOT_FOUND")], 100
    ),
    "robot_get_failover": operation(
        "GET", "/failover/{failover-ip}", [], "failover-detail",
        [(404, "NOT_FOUND")], 100,
    ),
    "robot_reroute_failover": operation(
        "POST", "/failover/{failover-ip}", ["active_server_ip"], "failover-detail",
        [
            (400, "INVALID_INPUT"),
            (404, "NOT_FOUND"),
            (404, "FAILOVER_NEW_SERVER_NOT_FOUND"),
            (409, "FAILOVER_ALREADY_ROUTED"),
            (409, "FAILOVER_LOCKED"),
            (500, "FAILOVER_FAILED"),
            (500, "FAILOVER_NOT_COMPLETE"),
        ],
        50,
    ),
    "robot_delete_failover_route": operation(
        "DELETE", "/failover/{failover-ip}", [], "failover-detail-null-route",
        [
            (404, "NOT_FOUND"),
            (409, "FAILOVER_LOCKED"),
            (500, "FAILOVER_FAILED"),
            (500, "FAILOVER_NOT_COMPLETE"),
        ],
        50,
    ),
}


def fail(message: str) -> None:
    raise SystemExit(f"Robot failover contract: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def read_lock() -> tuple[dict[str, Any], bytes]:
    try:
        payload = LOCK.read_bytes()
    except OSError as error:
        fail(f"could not read fixture: {error}")
    require(len(payload) <= MAX_LOCK_BYTES, "fixture exceeds 16 KiB")
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"fixture is invalid UTF-8 JSON: {error}")
    require(isinstance(value, dict), "fixture root must be an object")
    return value, payload


def validate_contract(value: dict[str, Any]) -> None:
    require(
        set(value) == {
            "schema_version", "source", "operations", "detail_fields",
            "source_inconsistencies", "policy",
        },
        "top-level fields changed",
    )
    require(value.get("schema_version") == 1, "schema version changed")
    require(
        value.get("source") == {
            "retrieved": "2026-08-12",
            "url": "https://robot.hetzner.com/doc/webservice/en.html",
            "sha256": SOURCE_SHA256,
        },
        "source identity changed",
    )
    operations = value.get("operations")
    require(isinstance(operations, list) and len(operations) == 4, "expected four operations")
    observed: dict[str, dict[str, Any]] = {}
    for item in operations:
        require(isinstance(item, dict), "operation must be an object")
        require(
            set(item) == {
                "id", "method", "path", "request_fields", "success", "errors", "quota",
            },
            "operation fields changed",
        )
        operation_id = item.get("id")
        require(isinstance(operation_id, str), "operation id must be text")
        require(operation_id not in observed, "duplicate operation id")
        observed[operation_id] = {key: item[key] for key in item if key != "id"}
    require(observed == EXPECTED_OPERATIONS, "complete operation contract changed")
    require(
        value.get("detail_fields") == [
            "ip", "netmask", "server_ip", "server_ipv6_net", "server_number",
            "active_server_ip",
        ],
        "detail fields changed",
    )
    require(
        value.get("source_inconsistencies") == {
            "active_server_ip_table_type": "string-but-delete-example-is-null",
            "delete_success_body": "json-failover-object-not-no-content",
        },
        "reviewed source inconsistency changed",
    )
    require(
        value.get("policy") == {
            "canonical_route_address": True,
            "contiguous_netmask": True,
            "route_host_bits": "reject",
            "route_destination_family": "exact-match",
            "unknown_fields": "reject",
            "duplicate_routes": "reject",
            "reroute_permit": "mutation",
            "delete_permit": "destructive",
            "reroute_body": "sensitive-form",
            "reroute_retry": "never",
            "delete_retry": "never",
            "reroute_outcome": "exact-active-server",
            "delete_outcome": "active-server-null",
            "success_body_bytes": {"list": 2097152, "item": 16384},
            "list_item_boundary": [4095, 4096, 4097],
        },
        "security policy changed",
    )


def validate_implementation_policy() -> None:
    try:
        prepare = PREPARE_SOURCE.read_text(encoding="utf-8")
        decode = DECODE_SOURCE.read_text(encoding="utf-8")
        exchange = EXCHANGE_SOURCE.read_text(encoding="utf-8")
    except OSError as error:
        fail(f"could not read implementation policy source: {error}")
    for token in [
        "RequestSemantics::NonIdempotent",
        "RetryEligibility::Never",
        "RequestBodySensitivity::Sensitive",
        "RobotFailoverDestructivePermit",
    ]:
        source = prepare if token != "RobotFailoverDestructivePermit" else (
            ROOT / "crates/cloud-sdk-hetzner/src/robot/failover/permit.rs"
        ).read_text(encoding="utf-8")
        require(token in source, f"implementation lost {token}")
    require("contiguous_u32" in decode and "contiguous_u128" in decode, "netmask checks changed")
    for token in [
        "require_body_limit(checked, MAX_ROBOT_FAILOVER_LIST_RESPONSE_BYTES)",
        "require_body_limit(checked, MAX_ROBOT_FAILOVER_ITEM_RESPONSE_BYTES)",
        "RobotFailoverDecodeError::ResponseTooLarge",
    ]:
        require(token in decode, f"decoder lost independent bound: {token}")
    require("MutationOutcomeMismatch" in exchange, "exact mutation outcomes are not enforced")
    require("actual.is_none()" in exchange, "delete JSON null acknowledgement is not enforced")


def validate_fuzz_policy() -> None:
    try:
        source = FUZZ_SOURCE.read_text(encoding="utf-8")
        harness = FUZZ_HARNESS.read_text(encoding="utf-8")
    except OSError as error:
        fail(f"could not read fuzz policy source: {error}")
    for boundary in ["2_097_151", "2_097_152", "2_097_153"]:
        require(boundary in source, f"fuzz target lost {boundary}-byte boundary")
    require(
        'elif [ "$target" = robot_failover_response ]; then\n'
        "            # One selector byte plus the complete 2 MiB list-response boundary.\n"
        "            max_len=2097153" in harness,
        "fuzz smoke limit no longer reaches the complete failover list boundary",
    )
    expected = {
        "list-boundary-minus-one.seed": b"0B-\n",
        "list-boundary-exact.seed": b"0B0\n",
        "list-boundary-plus-one.seed": b"0B+\n",
    }
    for name, payload in expected.items():
        try:
            actual = (FUZZ_SEEDS / name).read_bytes()
        except OSError as error:
            fail(f"could not read {name}: {error}")
        require(actual == payload, f"deterministic boundary seed changed: {name}")


def main() -> None:
    value, payload = read_lock()
    require(hashlib.sha256(payload).hexdigest() == LOCK_SHA256, "fixture digest changed")
    validate_contract(value)
    validate_implementation_policy()
    validate_fuzz_policy()
    print("7 Robot failover source and fuzz policies passed; compiled tests enforce bounds.")


if __name__ == "__main__":
    main()
