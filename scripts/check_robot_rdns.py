#!/usr/bin/env python3
"""Validate the v0.86 Robot reverse-DNS source lock and implementation policy."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "tests/fixtures/robot-rdns/v0.86.0.json"
LOCK_SHA256 = "e081bdb2e49bd005955e66341847497efbb1000dbec2b6c756ad43259fe7c753"
SOURCE_SHA256 = "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a"
MAX_LOCK_BYTES = 16 * 1024
RDNS_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/rdns"

OPERATIONS = [
    ("robot_list_rdns", "GET", "/rdns", [200], ["server_ip?"], "rdns-list"),
    ("robot_get_rdns", "GET", "/rdns/{ip}", [200], [], "rdns"),
    ("robot_set_rdns", "PUT", "/rdns/{ip}", [201], ["ptr"], "rdns"),
    ("robot_update_rdns", "POST", "/rdns/{ip}", [200, 201], ["ptr"], "rdns"),
    ("robot_delete_rdns", "DELETE", "/rdns/{ip}", [200], [], "empty"),
]

ERRORS = {
    "robot_list_rdns": [[404, "NOT_FOUND"]],
    "robot_get_rdns": [[404, "IP_NOT_FOUND"], [404, "RDNS_NOT_FOUND"]],
    "robot_set_rdns": [[400, "INVALID_INPUT"], [404, "IP_NOT_FOUND"], [409, "RDNS_ALREADY_EXISTS"], [500, "RDNS_CREATE_FAILED"]],
    "robot_update_rdns": [[400, "INVALID_INPUT"], [404, "IP_NOT_FOUND"], [500, "RDNS_CREATE_FAILED"], [500, "RDNS_UPDATE_FAILED"]],
    "robot_delete_rdns": [[404, "IP_NOT_FOUND"], [500, "RDNS_DELETE_FAILED"], [500, "RDNS_UPDATE_FAILED"]],
}

POLICY = {
    "address_identity": "canonical-ipv4-or-ipv6",
    "server_filter": "canonical-ipv4-main-address-only",
    "ptr_identity": "lowercase-ascii-dns-name-without-root-dot",
    "ptr_bytes": 253,
    "unknown_fields": "reject",
    "response_identity": "exact-request-address",
    "mutation_outcome": "exact-request-ptr",
    "list_identity": "distinct-addresses",
    "maximum_list_items": 4096,
    "list_response_bytes": 2097152,
    "item_response_bytes": 16384,
    "delete_response": "empty",
    "mutation_retry": "never",
    "mutation_authority": "request-bound-permit",
    "destructive_authority": "request-bound-permit",
}


def fail(message: str) -> None:
    raise SystemExit(f"Robot reverse-DNS contract: {message}")


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
    require(set(value) == {"schema_version", "source", "operations", "quota", "policy"}, "top-level fields changed")
    require(value["schema_version"] == 1, "schema version changed")
    require(value["source"] == {"retrieved": "2026-08-13", "url": "https://robot.hetzner.com/doc/webservice/en.html", "sha256": SOURCE_SHA256}, "source identity changed")
    require(value["quota"] == {"requests": 500, "seconds": 3600}, "quota changed")
    require(value["policy"] == POLICY, "security policy changed")
    operations = value.get("operations")
    require(isinstance(operations, list) and len(operations) == 5, "expected five operations")
    for item, expected in zip(operations, OPERATIONS, strict=True):
        require(set(item) == {"id", "method", "path", "success", "input", "output", "errors"}, "operation fields changed")
        observed = tuple(item[field] for field in ("id", "method", "path", "success", "input", "output"))
        require(observed == expected, f"operation changed: {expected[0]}")
        require(item["errors"] == ERRORS[expected[0]], f"operation errors changed: {expected[0]}")
    ids = [item["id"] for item in operations]
    require(len(ids) == len(set(ids)), "duplicate operation id")


def validate_implementation_policy() -> None:
    sources = {
        path.relative_to(RDNS_SOURCE).as_posix(): path.read_text(encoding="utf-8")
        for path in RDNS_SOURCE.rglob("*.rs")
    }
    required = {
        "prepare.rs": ["/rdns", "StatusCode::CREATED", "ResponseBodyPolicy::Forbidden", "RetryEligibility::Never", "RobotFormField::sensitive(\"ptr\""],
        "request.rs": ["InvalidServerAddress", "RobotRdnsSetRequest", "RobotRdnsUpdateRequest", "RobotRdnsDeleteRequest"],
        "value.rs": ["MAX_ROBOT_RDNS_NAME_BYTES", "valid_label", "SecretBoxBytes", "constant_time_eq"],
        "decode.rs": ["MAX_ROBOT_RDNS_LIST_ITEMS", "reject_duplicates_by_cmp", "ResponseIdentityMismatch", "InvalidPtr"],
        "exchange.rs": ["MutationOutcomeMismatch", "RobotRdnsDeleteRequest", "decode_robot_rdns"],
        "failure.rs": ["RDNS_ALREADY_EXISTS", "RDNS_CREATE_FAILED", "RDNS_UPDATE_FAILED", "RDNS_DELETE_FAILED"],
        "permit.rs": ["RobotRdnsSetRequest", "RobotRdnsUpdateRequest", "RobotRdnsDeleteRequest"],
    }
    for name, tokens in required.items():
        require(name in sources, f"implementation lost {name}")
        for token in tokens:
            require(token in sources[name], f"implementation lost {name}: {token}")
    combined = "\n".join(sources.values())
    for forbidden in ["RetryEligibility::Always", "RequestSemantics::Safe,\n            RetryEligibility::Never"]:
        require(forbidden not in combined, f"unsafe request policy admitted: {forbidden}")


def main() -> None:
    value, payload = read_lock()
    require(hashlib.sha256(payload).hexdigest() == LOCK_SHA256, "fixture digest changed")
    validate_contract(value)
    validate_implementation_policy()
    print("5 Robot reverse-DNS operations and 14 source/security policy groups passed.")


if __name__ == "__main__":
    main()
