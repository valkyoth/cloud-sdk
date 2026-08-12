#!/usr/bin/env python3
"""Validate the v0.82 Robot reset source lock."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "tests/fixtures/robot-reset/v0.82.0.json"
LOCK_SHA256 = "a51e48739b69afd3f27290a2daaf360951e565667234700c5325d1be5a475444"
MAX_LOCK_BYTES = 16 * 1024
SOURCE_SHA256 = "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a"
PREPARE_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/reset/prepare.rs"
EXCHANGE_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/reset/exchange.rs"
CORE_PREPARED_SOURCE = ROOT / "crates/cloud-sdk/src/operation/prepared.rs"
CORE_VALIDATION_SOURCE = (
    ROOT / "crates/cloud-sdk/src/operation/permit/fingerprint/validation.rs"
)


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
    "robot_list_resets": operation(
        "GET", "/reset", [], "reset-list", [(404, "NOT_FOUND")], 500
    ),
    "robot_get_reset": operation(
        "GET", "/reset/{server-number}", [], "reset-detail",
        [(404, "SERVER_NOT_FOUND"), (404, "RESET_NOT_AVAILABLE")], 500,
    ),
    "robot_execute_reset": operation(
        "POST", "/reset/{server-number}", ["type"], "reset-action",
        [
            (400, "INVALID_INPUT"),
            (404, "SERVER_NOT_FOUND"),
            (404, "RESET_NOT_AVAILABLE"),
            (409, "RESET_MANUAL_ACTIVE"),
            (500, "RESET_FAILED"),
        ],
        50,
    ),
}


def fail(message: str) -> None:
    raise SystemExit(f"Robot reset contract: {message}")


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
            "reset_types", "source_inconsistencies", "policy",
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
    require(isinstance(operations, list) and len(operations) == 3, "expected three operations")
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
        observed[operation_id] = {key: value for key, value in item.items() if key != "id"}
    require(observed == EXPECTED_OPERATIONS, "complete operation contract changed")
    require(
        value.get("detail_fields") == [
            "server_ip", "server_ipv6_net", "server_number", "type", "operating_status",
        ],
        "detail fields changed",
    )
    require(
        value.get("reset_types") == ["sw", "hw", "power", "power_long", "man"],
        "reset type set changed",
    )
    require(
        value.get("source_inconsistencies") == {
            "post_server_number": "required-by-output-table-omitted-by-example",
            "deprecated_server_ip_route": "excluded",
        },
        "reviewed source inconsistency changed",
    )
    require(
        value.get("policy") == {
            "canonical_server_number_route": True,
            "unknown_fields": "reject",
            "capabilities": "nonempty-finite-duplicate-free",
            "execute_from_authenticated_detail": True,
            "credential_lineage": "opaque-transport-binding",
            "preflight_evidence_seconds": 30,
            "dispatch_revalidation": "credential-and-expiry",
            "generic_type_erasure": "forbidden-by-provider-and-core",
            "execute_permit": "destructive",
            "execute_body": "sensitive-form",
            "execute_retry": "never",
            "action_identity": "bind-ipv4-ipv6-and-optional-number",
            "action_type": "exact-request-match",
            "success_body_bytes": {"list": 2097152, "detail": 4096, "action": 2048},
            "list_item_boundary": [4095, 4096, 4097],
        },
        "security policy changed",
    )


def validate_implementation_policy() -> None:
    try:
        prepare = PREPARE_SOURCE.read_text(encoding="utf-8")
        exchange = EXCHANGE_SOURCE.read_text(encoding="utf-8")
        core_prepared = CORE_PREPARED_SOURCE.read_text(encoding="utf-8")
        core_validation = CORE_VALIDATION_SOURCE.read_text(encoding="utf-8")
    except OSError as error:
        fail(f"could not read implementation policy source: {error}")
    require(
        "impl PrepareOperation for RobotResetExecuteRequest" not in prepare,
        "execute request entered generic PrepareOperation",
    )
    require(
        "prepared.with_required_authorization_evidence()" in prepare,
        "execute preparation lost the core evidence marker",
    )
    require(
        exchange.count("pub const fn as_untyped") == 2
        and "RobotResetListRequest>" in exchange
        and "RobotResetGetRequest>" in exchange,
        "execute wrapper exposes generic prepared request",
    )
    require(
        "with_required_authorization_evidence" in core_prepared,
        "core prepared request lost the evidence marker",
    )
    require(
        "AuthorizationEvidenceRequired" in core_validation,
        "generic plan validation no longer rejects missing evidence",
    )


def main() -> None:
    value, payload = read_lock()
    require(hashlib.sha256(payload).hexdigest() == LOCK_SHA256, "fixture digest changed")
    validate_contract(value)
    validate_implementation_policy()
    print("4 Robot reset source policies passed; compiled tests enforce implementation policy.")


if __name__ == "__main__":
    main()
