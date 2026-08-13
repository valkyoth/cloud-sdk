#!/usr/bin/env python3
"""Validate the immutable v0.86 Robot reverse-DNS source contract."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "tests/fixtures/robot-rdns/v0.86.0.json"
LOCK_SHA256 = "0baecb5e6b4db7dcc63326eb49a5b588d49c4f8b8b8be7238c3f62b8e98b0717"
SOURCE_SHA256 = "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a"
MAX_LOCK_BYTES = 16 * 1024

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
    "filtered_response_identity": "checked-ip-inventory-required",
    "filtered_response_result": "non-empty-membership-only",
    "filtered_response_lookup": "sorted-assignment-index",
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
    "mutation_semantics": "non-idempotent",
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


def main() -> None:
    value, payload = read_lock()
    require(hashlib.sha256(payload).hexdigest() == LOCK_SHA256, "fixture digest changed")
    validate_contract(value)
    print("5 Robot reverse-DNS operations and exact source contract passed.")


if __name__ == "__main__":
    main()
