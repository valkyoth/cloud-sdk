#!/usr/bin/env python3
"""Validate immutable v0.93.0 Robot billable-order evidence."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests/fixtures/robot-order-mutations/v0.93.0.json"
API_LOCK = ROOT / "tests/fixtures/robot-api/v0.74.0.json"
MODULE = ROOT / "crates/cloud-sdk-hetzner/src/robot/ordering/mutation"

OPERATIONS = [
    ("robot_create_server_transaction", "create_server_transaction", "/order/server/transaction"),
    ("robot_create_server_market_transaction", "create_server_market_transaction", "/order/server_market/transaction"),
    ("robot_create_server_addon_transaction", "create_server_addon_transaction", "/order/server_addon/transaction"),
]


def fail(message: str) -> None:
    raise SystemExit(f"Robot order mutation contract: {message}")


def load(path: Path) -> dict:
    try:
        value = json.loads(path.read_text(encoding="ascii"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        fail(f"cannot read {path}: {error}")
    if not isinstance(value, dict):
        fail(f"{path} root is not an object")
    return value


def validate(fixture_path: Path, api_lock_path: Path, module_path: Path) -> None:
    fixture = load(fixture_path)
    if fixture.get("schema_version") != 1:
        fail("unexpected fixture schema")
    source = fixture.get("source", {})
    if source != {
        "retrieved": "2026-08-16",
        "url": "https://robot.hetzner.com/doc/webservice/en.html",
        "sha256": "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a",
    }:
        fail("official source lock changed")
    validate_operations(fixture, api_lock_path)
    validate_policy(fixture)
    validate_implementation(module_path)
    validate_nonexecution()


def validate_operations(fixture: dict, api_lock_path: Path) -> None:
    operations = fixture.get("operations")
    if not isinstance(operations, list) or len(operations) != 3:
        fail("exactly three active billable operations are required")
    inventory = {
        row.get("id"): row
        for row in load(api_lock_path).get("operations", [])
        if row.get("group") == "ordering_mutation"
    }
    if set(inventory) != {item[1] for item in OPERATIONS}:
        fail("ordering-mutation inventory membership changed")
    for index, (operation_id, inventory_id, path) in enumerate(OPERATIONS):
        row = operations[index]
        expected_errors = [[400, "INVALID_INPUT"], [412, "PRECONDITION_FAILED"], [500, "INTERNAL_ERROR"]]
        if inventory_id == "create_server_addon_transaction":
            expected_errors.insert(1, [409, "CONFLICT"])
        expected = {
            "id": operation_id,
            "inventory_id": inventory_id,
            "method": "POST",
            "path": path,
            "success": 201,
            "quota": {"requests": 20, "interval_seconds": 86400},
            "errors": expected_errors,
        }
        if row != expected:
            fail(f"operation contract changed for {inventory_id}")
        locked = inventory[inventory_id]
        if locked.get("method") != "POST" or locked.get("path") != path:
            fail(f"API inventory drifted for {inventory_id}")
        if locked.get("status") != "active" or locked.get("milestone") != "v0.93.0":
            fail(f"API inventory status drifted for {inventory_id}")


def validate_policy(fixture: dict) -> None:
    if fixture.get("request") != {
        "authentication": "robot-basic",
        "content_type": "application/x-www-form-urlencoded",
        "standard_fields": ["product_id", "dist", "lang", "location", "addon[]"],
        "market_fields": ["product_id", "dist", "lang"],
        "addon_fields": ["server_number", "product_id"],
        "deprecated_fields": ["arch"],
    }:
        fail("request-field policy changed")
    if fixture.get("local_policy") != {
        "cost": "catalog-gross-recurring-plus-setup-scale-4",
        "account": "required-fingerprint-scope",
        "response_bytes": 1048576,
        "automatic_retry": "never",
        "single_attempt": "no-idempotency-key",
        "uncertain_repeat": "fresh-exact-plan-plus-same-idempotency-plus-absent-transaction-proof",
        "ci_purchase": False,
    }:
        fail("local billable-operation policy changed")


def validate_implementation(module: Path) -> None:
    source = "\n".join(path.read_text(encoding="ascii") for path in module.rglob("*.rs"))
    prepare = (module / "prepare.rs").read_text(encoding="ascii")
    permit = (module / "permit.rs").read_text(encoding="ascii")
    reconcile = (module / "reconcile.rs").read_text(encoding="ascii")
    for operation_id, _, path in OPERATIONS:
        if operation_id not in prepare or path not in prepare:
            fail(f"implementation lost {operation_id}")
    for token in [
        "OperationImpact::Mutation",
        "RequestSemantics::NonIdempotent",
        "RetryEligibility::Never",
        "CostIntent::MayIncurCost",
        "StatusCode::CREATED",
        "RequestBodySensitivity::Sensitive",
        "MAX_ROBOT_FORM_FIELDS",
    ]:
        if token not in prepare:
            fail(f"preparation policy lost {token}")
    for token in [
        "RobotOrderCostPermit",
        "Some(request.plan_cost())",
        "RobotOrderAccount",
        "PermitValidity",
        "ReplayPolicy",
        "RobotOrderNotApplied",
        "reconcile_not_applied",
        "core::ptr::eq",
    ]:
        if token not in permit:
            fail(f"cost authority lost {token}")
    if permit.count("core::ptr::eq") != 2:
        fail("request/proof identity checks changed")
    for token in ["MatchingTransaction", ".any(|value| self.matches_transaction(value))"]:
        if token not in reconcile:
            fail(f"reconciliation lost {token}")
    if reconcile.count("MatchingTransaction") != 5:
        fail("matching-transaction rejection coverage changed")
    if "impl PrepareOperation for" in source or "pub fn as_untyped" in source:
        fail("billable request regained an unguarded or type-erased execution route")


def validate_nonexecution() -> None:
    paths = list((ROOT / ".github").rglob("*")) + [
        ROOT / "scripts/hetzner-live-smoke.py",
        ROOT / "scripts/hetzner-live-smoke-runner.py",
    ]
    forbidden = [path for _, _, path in OPERATIONS]
    for path in paths:
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        if any(route in text for route in forbidden):
            fail(f"CI/live harness gained a billable Robot route: {path.relative_to(ROOT)}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture", type=Path, default=FIXTURE)
    parser.add_argument("--api-lock", type=Path, default=API_LOCK)
    parser.add_argument("--module", type=Path, default=MODULE)
    args = parser.parse_args()
    validate(args.fixture, args.api_lock, args.module)
    print("3 Robot billable-order contracts and CI non-execution passed.")


if __name__ == "__main__":
    main()
