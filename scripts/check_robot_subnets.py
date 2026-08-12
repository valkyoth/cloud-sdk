#!/usr/bin/env python3
"""Validate the v0.81 Robot subnet source lock."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "tests/fixtures/robot-subnet/v0.81.0.json"
LOCK_SHA256 = "286c7e7462dbbaaccb3864c175ef38c32901b8b7333ce8c70bef7d3baea03952"
MAX_LOCK_BYTES = 32 * 1024
SOURCE_SHA256 = "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a"

OPERATIONS = {
    "robot_list_subnets": ("GET", "/subnet", "subnet-list"),
    "robot_get_subnet": ("GET", "/subnet/{net-ip}", "subnet-detail"),
    "robot_update_subnet": ("POST", "/subnet/{net-ip}", "subnet-detail"),
    "robot_get_subnet_mac": ("GET", "/subnet/{net-ip}/mac", "subnet-mac"),
    "robot_set_subnet_mac": ("PUT", "/subnet/{net-ip}/mac", "subnet-mac"),
    "robot_delete_subnet_mac": ("DELETE", "/subnet/{net-ip}/mac", "subnet-mac"),
}


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
            operation.get("method"), operation.get("path"), success.get("shape")
        )
    require(observed == OPERATIONS, "operation route or shape policy changed")
    require(len(value.get("subnet_fields", [])) == 11, "subnet fields changed")
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
