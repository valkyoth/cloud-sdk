#!/usr/bin/env python3
"""Validate immutable v0.92.0 Robot transaction evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests/fixtures/robot-transactions/v0.92.0.json"
API_LOCK = ROOT / "tests/fixtures/robot-api/v0.74.0.json"
MODULE = ROOT / "crates/cloud-sdk-hetzner/src/robot/ordering/transaction"
README = ROOT / "crates/cloud-sdk-hetzner/README.md"
FUZZ_MANIFEST = ROOT / "fuzz/Cargo.toml"
FUZZ_GATE = ROOT / "scripts/check_fuzz_harness.sh"

OPERATIONS = [
    ("robot_list_server_transactions", "list_server_transactions", "/order/server/transaction", 30, "json-list"),
    ("robot_get_server_transaction", "get_server_transaction", "/order/server/transaction/{id}", None, "json-detail"),
    ("robot_list_server_market_transactions", "list_server_market_transactions", "/order/server_market/transaction", 30, "json-list"),
    ("robot_get_server_market_transaction", "get_server_market_transaction", "/order/server_market/transaction/{id}", None, "json-detail"),
    ("robot_list_server_addon_transactions", "list_server_addon_transactions", "/order/server_addon/transaction", 30, "json-list"),
    ("robot_get_server_addon_transaction", "get_server_addon_transaction", "/order/server_addon/transaction/{id}", None, "json-detail"),
]


def fail(message: str) -> None:
    raise SystemExit(f"Robot transaction contract: {message}")


def load(path: Path) -> dict:
    try:
        value = json.loads(path.read_text(encoding="ascii"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        fail(f"cannot read {path}: {error}")
    if not isinstance(value, dict):
        fail(f"{path} root is not an object")
    return value


def validate(
    fixture_path: Path,
    api_lock_path: Path,
    module_path: Path,
    readme_path: Path,
    fuzz_manifest_path: Path,
    fuzz_gate_path: Path,
) -> None:
    fixture = load(fixture_path)
    if fixture.get("schema_version") != 1:
        fail("unexpected fixture schema")
    source = fixture.get("source", {})
    if source.get("url") != "https://robot.hetzner.com/doc/webservice/en.html":
        fail("official source URL changed")
    if source.get("sha256") != "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a":
        fail("official source digest changed without review")
    validate_examples(source.get("examples"))
    expected = [
        {
            "id": operation_id,
            "inventory_id": inventory_id,
            "method": "GET",
            "path": path,
            "success": 200,
            "quota": {"requests": 500, "interval_seconds": 3600},
            "window_days": window,
            "body": body,
        }
        for operation_id, inventory_id, path, window, body in OPERATIONS
    ]
    if fixture.get("operations") != expected:
        fail("six-operation transaction contract changed")
    validate_inventory(api_lock_path)
    validate_schema(fixture)
    validate_sources(module_path, readme_path, fuzz_manifest_path, fuzz_gate_path)


def validate_examples(examples: object) -> None:
    if not isinstance(examples, list) or len(examples) != 3:
        fail("three reviewed response examples are required")
    for item in examples:
        if not isinstance(item, dict):
            fail("response example metadata is malformed")
        relative = item.get("path")
        expected = item.get("sha256")
        if not isinstance(relative, str) or not isinstance(expected, str):
            fail("response example metadata is incomplete")
        try:
            actual = hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()
        except OSError as error:
            fail(f"cannot read response example {relative}: {error}")
        if actual != expected:
            fail(f"response example digest changed: {relative}")


def validate_inventory(path: Path) -> None:
    values = load(path).get("operations", [])
    selected = {item.get("id"): item for item in values if item.get("group") == "ordering_transaction"}
    expected_ids = {operation[1] for operation in OPERATIONS}
    if set(selected) != expected_ids:
        fail("API inventory transaction membership changed")
    for _, inventory_id, route, _, _ in OPERATIONS:
        item = selected[inventory_id]
        if item.get("method") != "GET" or item.get("path") != route:
            fail(f"API inventory drifted for {inventory_id}")
        if item.get("status") != "active" or item.get("milestone") != "v0.92.0":
            fail(f"API inventory status drifted for {inventory_id}")


def validate_schema(fixture: dict) -> None:
    if fixture.get("request") != {
        "authentication": "robot-basic",
        "content_type": None,
        "pagination": "none",
    }:
        fail("transaction request schema changed")
    if fixture.get("response") != {
        "content_type": "application/json",
        "statuses": ["ready", "in process", "cancelled"],
        "server_fields": ["id", "date", "status", "server_number", "server_ip", "authorized_key", "host_key", "comment", "product"],
        "standard_extra_fields": ["addons"],
        "addon_fields": ["id", "date", "status", "server_number", "product", "resources"],
        "failure": {"status": 404, "code": "NOT_FOUND"},
    }:
        fail("transaction response schema changed")
    if fixture.get("local_policy") != {
        "transaction_items": 4096,
        "keys_per_transaction": 64,
        "resources_per_transaction": 4096,
        "list_response_bytes": 4194304,
        "item_response_bytes": 1048576,
        "automatic_retry": "explicit-policy",
        "ready_server_identity": "required",
        "non_ready_server_identity": "forbidden",
        "detail_identity": "request-bound",
    }:
        fail("transaction local policy changed")


def validate_sources(
    module: Path, readme: Path, fuzz_manifest: Path, fuzz_gate: Path
) -> None:
    files = {path.name: path.read_text(encoding="ascii") for path in module.glob("*.rs")}
    nested = "\n".join(path.read_text(encoding="ascii") for path in module.rglob("*.rs"))
    prepare = files.get("prepare.rs", "")
    for operation_id, _, path, _, _ in OPERATIONS:
        if operation_id not in prepare or path.split("{")[0].rstrip("/") not in prepare:
            fail(f"implementation lost {operation_id}")
    for token in ["OperationImpact::ReadOnly", "RequestSemantics::Safe", "RetryEligibility::ExplicitPolicy", "CostIntent::NoKnownCost"]:
        if token not in prepare:
            fail(f"preparation policy lost {token}")
    if "Method::Post" in prepare or "Method::Delete" in prepare:
        fail("transaction reads expose a mutation method")
    for token in ["RobotOrderTransactionStatus::Ready", "valid_rfc3339", "reject_transaction_duplicates", "reject_duplicates_by_cmp"]:
        if token not in nested:
            fail(f"strict transaction evidence lost {token}")
    if files.get("exchange.rs", "").count("ResponseIdentityMismatch") != 3:
        fail("detail response identity binding changed")
    if "status == 404 && code == \"NOT_FOUND\"" not in files.get("failure.rs", ""):
        fail("source-locked NOT_FOUND decoder changed")
    if "all six active read-only transaction operations" not in readme.read_text(encoding="ascii"):
        fail("provider README lost transaction scope")
    manifest = fuzz_manifest.read_text(encoding="ascii")
    gate = fuzz_gate.read_text(encoding="ascii")
    if 'name = "robot_transaction_response"' not in manifest:
        fail("transaction response fuzz target is missing")
    if "passed for 34 targets" not in gate or gate.count("max_len=4194305") < 2:
        fail("transaction fuzz boundary is not source-locked")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture", type=Path, default=FIXTURE)
    parser.add_argument("--api-lock", type=Path, default=API_LOCK)
    parser.add_argument("--module", type=Path, default=MODULE)
    parser.add_argument("--readme", type=Path, default=README)
    parser.add_argument("--fuzz-manifest", type=Path, default=FUZZ_MANIFEST)
    parser.add_argument("--fuzz-gate", type=Path, default=FUZZ_GATE)
    args = parser.parse_args()
    validate(args.fixture, args.api_lock, args.module, args.readme, args.fuzz_manifest, args.fuzz_gate)
    print("Robot transaction source contract passed.")


if __name__ == "__main__":
    main()
