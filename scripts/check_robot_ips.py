#!/usr/bin/env python3
"""Validate the v0.80 Robot IP source and implementation lock."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "tests/fixtures/robot-ip/v0.80.0.json"
LOCK_SHA256 = "b37a0c6259eaae354b3e691db0974882823a5d008545d8bb4bcf6d2f69c54247"
MAX_LOCK_BYTES = 32 * 1024
SOURCE_SHA256 = "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a"

OPERATIONS = {
    "robot_list_ips": ("GET", "/ip", "ip-list"),
    "robot_get_ip": ("GET", "/ip/{ip}", "ip-detail"),
    "robot_update_ip": ("POST", "/ip/{ip}", "ip-detail"),
    "robot_get_ip_mac": ("GET", "/ip/{ip}/mac", "mac-present"),
    "robot_set_ip_mac": ("PUT", "/ip/{ip}/mac", "mac-present"),
    "robot_delete_ip_mac": ("DELETE", "/ip/{ip}/mac", "mac-null"),
}


def fail(message: str) -> None:
    raise SystemExit(f"Robot IP contract: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def read_lock() -> tuple[dict[str, Any], bytes]:
    try:
        payload = LOCK.read_bytes()
    except OSError as error:
        fail(f"could not read fixture: {error}")
    require(len(payload) <= MAX_LOCK_BYTES, "fixture exceeds 32 KiB")
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"fixture is invalid UTF-8 JSON: {error}")
    require(isinstance(value, dict), "fixture root must be an object")
    return value, payload


def validate_contract(value: dict[str, Any]) -> None:
    require(
        set(value) == {
            "schema_version", "source", "operations", "summary_fields",
            "detail_fields", "mac_fields", "policy",
        },
        "top-level fields changed",
    )
    require(value.get("schema_version") == 1, "schema version changed")
    source = value.get("source")
    require(isinstance(source, dict), "source identity is missing")
    require(
        source.get("url") == "https://robot.hetzner.com/doc/webservice/en.html"
        and source.get("sha256") == SOURCE_SHA256,
        "source identity changed",
    )
    operations = value.get("operations")
    require(isinstance(operations, list) and len(operations) == 6, "expected six operations")
    observed: dict[str, tuple[str, str, str]] = {}
    for operation in operations:
        require(isinstance(operation, dict), "operation must be an object")
        require(
            set(operation) == {
                "id", "method", "path", "request_fields", "success", "errors", "quota",
            },
            "operation fields changed",
        )
        operation_id = operation.get("id")
        success = operation.get("success")
        quota = operation.get("quota")
        require(isinstance(operation_id, str), "operation id must be text")
        require(
            isinstance(success, dict)
            and success.get("status") == 200
            and success.get("body") == "json",
            f"success policy changed for {operation_id}",
        )
        require(
            isinstance(quota, dict)
            and quota.get("seconds") == 3600
            and quota.get("requests") in {10, 5000},
            f"quota policy changed for {operation_id}",
        )
        observed[operation_id] = (
            operation.get("method"),
            operation.get("path"),
            success.get("shape"),
        )
    require(observed == OPERATIONS, "operation route or shape policy changed")
    require(len(value.get("summary_fields", [])) == 9, "summary fields changed")
    require(value.get("detail_fields") == ["gateway", "mask", "broadcast"],
            "detail fields changed")
    require(value.get("mac_fields") == ["ip", "mac"], "MAC fields changed")
    require(
        value.get("policy") == {
            "canonical_ip": True,
            "canonical_mac": "lowercase-eui48",
            "unknown_fields": "reject",
            "response_identity_binding": True,
            "non_empty_partial_update": True,
            "update_permit": "mutation",
            "set_mac_permit": "mutation",
            "delete_mac_permit": "destructive",
            "set_mac_retry": "never",
            "delete_mac_retry": "never",
        },
        "security policy changed",
    )


def validate_implementation() -> None:
    base = ROOT / "crates/cloud-sdk-hetzner/src/robot/ip"
    sources = "\n".join(path.read_text(encoding="utf-8") for path in sorted(base.rglob("*.rs")))
    for operation_id in OPERATIONS:
        require(f'"{operation_id}"' in sources, f"missing operation id {operation_id}")
    for required in (
        "PreparedRobotIp", "CheckedRobotIp", "ResponseIdentityMismatch",
        "MutationOutcomeMismatch", "MAX_ROBOT_IP_LIST_ITEMS", "RobotMacAddress",
        "RobotIpMutationPermit", "RobotIpDestructivePermit",
        "RobotIpSharedMutationPermit", "RobotIpSharedDestructivePermit",
        "RequestBodySensitivity::Sensitive", "OperationImpact::Destructive",
        "RequestSemantics::NonIdempotent", "RetryEligibility::Never",
        '"traffic_warnings"', '"traffic_hourly"', '"traffic_daily"',
        '"traffic_monthly"', "validate_network", "require_fields",
    ):
        require(required in sources, f"implementation policy missing {required}")


def main() -> None:
    value, payload = read_lock()
    require(hashlib.sha256(payload).hexdigest() == LOCK_SHA256, "fixture digest changed")
    validate_contract(value)
    validate_implementation()
    print("6 Robot IP operations and source policies passed.")


if __name__ == "__main__":
    main()
