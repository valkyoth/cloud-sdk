#!/usr/bin/env python3
"""Validate the v0.84 Robot Wake-on-LAN source lock."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "tests/fixtures/robot-wol/v0.84.0.json"
LOCK_SHA256 = "8bdd74948e05e8a3e539e7929d8d30a77e7562d9f31f4cec79cb32b405044f01"
MAX_LOCK_BYTES = 12 * 1024
SOURCE_SHA256 = "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a"
WOL_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/wol"


def operation(
    method: str,
    errors: list[tuple[int, str]],
    requests: int,
) -> dict[str, Any]:
    return {
        "method": method,
        "path": "/wol/{server-number}",
        "request_fields": [],
        "success": {"status": 200, "body": "json", "shape": "wol-identity"},
        "errors": [{"status": status, "code": code} for status, code in errors],
        "quota": {"requests": requests, "seconds": 3600},
    }


EXPECTED_OPERATIONS = {
    "robot_get_wol": operation(
        "GET", [(404, "SERVER_NOT_FOUND"), (404, "WOL_NOT_AVAILABLE")], 500
    ),
    "robot_send_wol": operation(
        "POST",
        [
            (404, "SERVER_NOT_FOUND"),
            (404, "WOL_NOT_AVAILABLE"),
            (500, "WOL_FAILED"),
        ],
        10,
    ),
}


def fail(message: str) -> None:
    raise SystemExit(f"Robot WOL contract: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def read_lock() -> tuple[dict[str, Any], bytes]:
    try:
        payload = LOCK.read_bytes()
    except OSError as error:
        fail(f"could not read fixture: {error}")
    require(len(payload) <= MAX_LOCK_BYTES, "fixture exceeds 12 KiB")
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"fixture is invalid UTF-8 JSON: {error}")
    require(isinstance(value, dict), "fixture root must be an object")
    return value, payload


def validate_contract(value: dict[str, Any]) -> None:
    require(
        set(value) == {
            "schema_version",
            "source",
            "operations",
            "identity_fields",
            "source_inconsistencies",
            "policy",
        },
        "top-level fields changed",
    )
    require(value.get("schema_version") == 1, "schema version changed")
    require(
        value.get("source")
        == {
            "retrieved": "2026-08-13",
            "url": "https://robot.hetzner.com/doc/webservice/en.html",
            "sha256": SOURCE_SHA256,
        },
        "source identity changed",
    )
    operations = value.get("operations")
    require(isinstance(operations, list) and len(operations) == 2, "expected two operations")
    observed: dict[str, dict[str, Any]] = {}
    for item in operations:
        require(isinstance(item, dict), "operation must be an object")
        require(
            set(item)
            == {"id", "method", "path", "request_fields", "success", "errors", "quota"},
            "operation fields changed",
        )
        operation_id = item.get("id")
        require(isinstance(operation_id, str), "operation id must be text")
        require(operation_id not in observed, "duplicate operation id")
        observed[operation_id] = {key: item[key] for key in item if key != "id"}
    require(observed == EXPECTED_OPERATIONS, "complete operation contract changed")
    require(
        value.get("identity_fields")
        == ["server_ip", "server_ipv6_net", "server_number"],
        "identity fields changed",
    )
    require(
        value.get("source_inconsistencies")
        == {
            "post_empty_form": "curl-demonstrates-empty-data-with-no-input-fields",
            "deprecated_server_ip_route": "excluded",
        },
        "source policy changed",
    )
    require(
        value.get("policy")
        == {
            "canonical_server_number_route": True,
            "unknown_fields": "reject",
            "address_families": "server-ipv4-and-ipv6-network",
            "send_intent": "explicit",
            "send_from_authenticated_discovery": True,
            "credential_lineage": "opaque-transport-binding",
            "preflight_evidence_seconds": 30,
            "dispatch_revalidation": "credential-and-expiry",
            "generic_type_erasure": "forbidden-by-provider-and-core",
            "send_permit": "mutation",
            "send_body": "empty-form",
            "send_retry": "never",
            "success_identity": "exact-server-number-ipv4-ipv6-network",
            "success_body_bytes": 16384,
        },
        "security policy changed",
    )


def validate_implementation_policy() -> None:
    sources = {
        name: (WOL_SOURCE / name).read_text(encoding="utf-8")
        for name in [
            "decode.rs",
            "evidence.rs",
            "exchange.rs",
            "failure.rs",
            "permit.rs",
            "prepare.rs",
            "request.rs",
        ]
    }
    requirements = {
        "prepare.rs": [
            'write_str(output, &mut len, "/wol/"',
            "OperationImpact::Mutation",
            "RequestSemantics::NonIdempotent",
            "RetryEligibility::Never",
            "ContentType::FORM_URLENCODED",
            "with_required_authorization_evidence",
        ],
        "decode.rs": [
            "MAX_ROBOT_WOL_RESPONSE_BYTES",
            "RobotWolDecodeError::ResponseTooLarge",
            "ResponseIdentityMismatch",
        ],
        "exchange.rs": ["same_identity(expected)", "ResponseIdentityMismatch"],
        "evidence.rs": [
            "MAX_ROBOT_WOL_EVIDENCE_AGE_SECONDS: u64 = 30",
            "CredentialChangedDuringPreflight",
        ],
        "permit.rs": [
            "RobotWolMutationPermit",
            "build_plan_digest_with_authorization_evidence",
            "validate_authorization_evidence",
        ],
        "request.rs": [
            "RobotWolIntent",
            "RobotWolSendRequest",
            "from_checked",
            "ROBOT_WOL_DISCOVERY_QUOTA",
            "max_requests: 500",
            "ROBOT_WOL_SEND_QUOTA",
            "max_requests: 10",
            "DelaySeconds::new(3_600)",
            "pub const fn quota(&self) -> RobotWolQuota",
        ],
        "failure.rs": ["WOL_NOT_AVAILABLE", "WOL_FAILED"],
    }
    for name, tokens in requirements.items():
        for token in tokens:
            require(token in sources[name], f"implementation lost {name}: {token}")
    combined = "\n".join(sources.values())
    for forbidden in ["/wol/{server-ip}", "RobotIpAddress)", "RetryEligibility::Always"]:
        require(forbidden not in combined, f"legacy or unsafe token admitted: {forbidden}")


def main() -> None:
    value, payload = read_lock()
    require(hashlib.sha256(payload).hexdigest() == LOCK_SHA256, "fixture digest changed")
    validate_contract(value)
    validate_implementation_policy()
    print("6 Robot WOL source, identity, capability, permit, and alias policies passed.")


if __name__ == "__main__":
    main()
