#!/usr/bin/env python3
"""Validate the v0.81 Robot subnet source lock."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "tests/fixtures/robot-subnet/v0.81.0.json"
LOCK_SHA256 = "98cb5d562d9a4ece8bba5045b7c9a439c34e4d8a476af16a4a1766f48e1c914d"
MAX_LOCK_BYTES = 32 * 1024
SOURCE_SHA256 = "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a"

def operation(
    method: str,
    path: str,
    request_fields: list[str],
    shape: str,
    errors: list[tuple[int, str]],
    requests: int,
) -> dict[str, Any]:
    return {
        "method": method,
        "path": path,
        "request_fields": request_fields,
        "success": {"status": 200, "body": "json", "shape": shape},
        "errors": [
            {"status": status, "code": code} for status, code in errors
        ],
        "quota": {"requests": requests, "seconds": 3600},
    }


EXPECTED_OPERATIONS = {
    "robot_list_subnets": operation(
        "GET", "/subnet", ["server_ip?"], "subnet-list",
        [(404, "NOT_FOUND")], 5000,
    ),
    "robot_get_subnet": operation(
        "GET", "/subnet/{net-ip}", [], "subnet-detail",
        [(404, "SUBNET_NOT_FOUND")], 5000,
    ),
    "robot_update_subnet": operation(
        "POST", "/subnet/{net-ip}",
        ["traffic_warnings?", "traffic_hourly?", "traffic_daily?", "traffic_monthly?"],
        "subnet-detail",
        [
            (400, "INVALID_INPUT"),
            (404, "SUBNET_NOT_FOUND"),
            (500, "TRAFFIC_WARNING_UPDATE_FAILED"),
        ],
        5000,
    ),
    "robot_get_subnet_mac": operation(
        "GET", "/subnet/{net-ip}/mac", [], "subnet-mac",
        [(404, "SUBNET_NOT_FOUND"), (404, "MAC_NOT_AVAILABLE")], 5000,
    ),
    "robot_set_subnet_mac": operation(
        "PUT", "/subnet/{net-ip}/mac", ["mac"], "subnet-mac",
        [
            (404, "SUBNET_NOT_FOUND"),
            (404, "MAC_NOT_AVAILABLE"),
            (500, "MAC_FAILED"),
        ],
        10,
    ),
    "robot_delete_subnet_mac": operation(
        "DELETE", "/subnet/{net-ip}/mac", [], "subnet-mac",
        [
            (404, "SUBNET_NOT_FOUND"),
            (404, "MAC_NOT_AVAILABLE"),
            (500, "MAC_FAILED"),
        ],
        10,
    ),
}

EXPECTED_SUBNET_FIELDS = [
    "ip", "mask", "gateway", "server_ip", "server_number", "failover",
    "locked", "traffic_warnings", "traffic_hourly", "traffic_daily",
    "traffic_monthly",
]


def fail(message: str) -> None:
    raise SystemExit(f"Robot subnet contract: {message}")


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
            "schema_version", "source", "operations", "subnet_fields",
            "mac_fields", "source_inconsistencies", "policy",
        },
        "top-level fields changed",
    )
    require(value.get("schema_version") == 1, "schema version changed")
    source = value.get("source")
    require(
        source == {
            "retrieved": "2026-08-12",
            "url": "https://robot.hetzner.com/doc/webservice/en.html",
            "sha256": SOURCE_SHA256,
        },
        "source identity changed",
    )
    operations = value.get("operations")
    require(isinstance(operations, list) and len(operations) == 6, "expected six operations")
    observed: dict[str, dict[str, Any]] = {}
    for operation in operations:
        require(isinstance(operation, dict), "operation must be an object")
        require(
            set(operation) == {
                "id", "method", "path", "request_fields", "success", "errors", "quota",
            },
            "operation fields changed",
        )
        operation_id = operation.get("id")
        require(isinstance(operation_id, str), "operation id must be text")
        require(operation_id not in observed, "duplicate operation id")
        observed[operation_id] = {
            key: item for key, item in operation.items() if key != "id"
        }
    require(observed == EXPECTED_OPERATIONS, "complete operation contract changed")
    require(value.get("subnet_fields") == EXPECTED_SUBNET_FIELDS, "subnet fields changed")
    require(
        value.get("mac_fields") == ["ip", "mask", "mac", "possible_mac"],
        "MAC fields changed",
    )
    require(
        value.get("source_inconsistencies") == {
            "server_ip": "table-string-example-null",
            "subnet_mask": "integer",
            "mac_mask": "decimal-string",
            "route_identity": "documented-examples-may-have-host-bits",
        },
        "reviewed source inconsistency changed",
    )
    require(
        value.get("policy") == {
            "canonical_ip": True,
            "canonical_mac": "lowercase-eui48",
            "unknown_fields": "reject",
            "response_identity_binding": True,
            "gateway_membership": True,
            "host_bits_set_identity": "admit",
            "possible_mac": "bounded-canonical-ip-to-mac-map",
            "non_empty_partial_update": True,
            "update_permit": "mutation",
            "set_mac_permit": "mutation",
            "delete_mac_permit": "destructive",
            "delete_mac_default_binding": "checked-subnet-server-to-possible-mac-digest-only",
            "delete_mac_observation_age_seconds": 30,
            "delete_mac_external_lock": "same-resource-protected-lease",
            "traffic_policy_storage": "redacted-non-copy-drop-cleared",
            "set_mac_retry": "never",
            "delete_mac_retry": "never",
        },
        "security policy changed",
    )


def main() -> None:
    value, payload = read_lock()
    require(hashlib.sha256(payload).hexdigest() == LOCK_SHA256, "fixture digest changed")
    validate_contract(value)
    print("6 Robot subnet source policies passed; compiled tests enforce implementation policy.")


if __name__ == "__main__":
    main()
