#!/usr/bin/env python3
"""Validate the v0.79 Robot cancellation source and implementation lock."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "tests/fixtures/robot-cancellation/v0.79.0.json"
LOCK_SHA256 = "588621f059b6cc73b39090223b25b7099efeb4f229bae44a8ec93febbe62d4a7"
MAX_LOCK_BYTES = 32 * 1024
SOURCE_SHA256 = "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a"

OPERATIONS = {
    "robot_get_server_cancellation": ("GET", "/server/{server-number}/cancellation"),
    "robot_create_server_cancellation": ("POST", "/server/{server-number}/cancellation"),
    "robot_delete_server_cancellation": ("DELETE", "/server/{server-number}/cancellation"),
    "robot_get_ip_cancellation": ("GET", "/ip/{ip}/cancellation"),
    "robot_create_ip_cancellation": ("POST", "/ip/{ip}/cancellation"),
    "robot_delete_ip_cancellation": ("DELETE", "/ip/{ip}/cancellation"),
    "robot_get_subnet_cancellation": ("GET", "/subnet/{net-ip}/cancellation"),
    "robot_create_subnet_cancellation": ("POST", "/subnet/{net-ip}/cancellation"),
    "robot_delete_subnet_cancellation": ("DELETE", "/subnet/{ip}/cancellation"),
}


def fail(message: str) -> None:
    raise SystemExit(f"Robot cancellation contract: {message}")


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
            "schema_version", "source", "operations", "server_response",
            "ip_response", "subnet_response", "policy", "source_inconsistencies",
        },
        "top-level fields changed",
    )
    require(value.get("schema_version") == 1, "schema version changed")
    require(
        value.get("source") == {
            "retrieved": "2026-08-10",
            "url": "https://robot.hetzner.com/doc/webservice/en.html",
            "sha256": SOURCE_SHA256,
        },
        "source identity changed",
    )
    operations = value.get("operations")
    require(isinstance(operations, list) and len(operations) == 9, "expected nine operations")
    observed: dict[str, tuple[str, str]] = {}
    for operation in operations:
        require(isinstance(operation, dict), "operation must be an object")
        require(
            set(operation) == {"id", "method", "path", "request_fields", "success", "errors"},
            "operation fields changed",
        )
        operation_id = operation.get("id")
        require(isinstance(operation_id, str), "operation id must be text")
        observed[operation_id] = (operation.get("method"), operation.get("path"))
        success = operation.get("success")
        require(isinstance(success, dict) and success.get("status") == 200, "success policy changed")
        if operation_id == "robot_delete_server_cancellation":
            require(success == {"status": 200, "body": "empty", "envelope": None},
                    "server DELETE success must stay empty")
        else:
            require(success == {"status": 200, "body": "json", "envelope": "cancellation"},
                    "JSON success envelope changed")
    require(observed == OPERATIONS, "operation route policy changed")
    require(
        value.get("policy") == {
            "create_impact": "destructive", "create_retry": "never",
            "delete_impact": "destructive", "delete_retry": "never",
            "date_input": ["now", "YYYY-MM-DD"], "response_identity_binding": True,
            "unknown_fields": "reject",
        },
        "safety policy changed",
    )
    for key in ("ip_response", "subnet_response"):
        require(value[key].get("date_field_variants") == ["cancellation_date", "cancellation-date"],
                f"{key} date variants changed")
    require(len(value.get("source_inconsistencies", [])) == 3, "source inconsistency record changed")


def validate_implementation() -> None:
    base = ROOT / "crates/cloud-sdk-hetzner/src/robot/cancellation"
    sources = "\n".join(path.read_text(encoding="utf-8") for path in sorted(base.glob("*.rs")))
    for operation_id in OPERATIONS:
        require(f'"{operation_id}"' in sources, f"missing operation id {operation_id}")
    for required in (
        "OperationImpact::Destructive", "RetryEligibility::Never",
        "Kind::Delete(Target::Server(_))", "ResponseBodyPolicy::Forbidden",
        "ResponseMediaPolicy::Forbidden",
        '"cancellation-date"', "ResponseIdentityMismatch", "reservation_possible",
        "protected_parse::subnet", "RequestBodySensitivity::Sensitive",
        "PreparedCancellation", "CheckedCancellation", "MutationOutcomeMismatch",
        "validate_schedule", "validate_reservation", "validate_reason",
        "map_date_error", "is_unsafe_display_character",
    ):
        require(required in sources, f"implementation policy missing {required}")


def main() -> None:
    value, payload = read_lock()
    require(hashlib.sha256(payload).hexdigest() == LOCK_SHA256, "fixture digest changed")
    validate_contract(value)
    validate_implementation()
    print("9 Robot cancellation operations and source policies passed.")


if __name__ == "__main__":
    main()
