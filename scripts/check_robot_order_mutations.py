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
        "addon_fields": ["server_number", "product_id", "reason", "gateway"],
        "addon_reason_required_types": ["ip_ipv4", "subnet_ipv4", "failover_subnet_ipv4"],
        "addon_gateway_type": "subnet_ipv4",
        "deprecated_fields": ["arch"],
    }:
        fail("request-field policy changed")
    if fixture.get("responses") != {
        "market_created_fields": ["id", "date", "status", "server_number", "server_ip", "authorized_key", "host_key", "comment", "product", "addons"],
        "market_created_product_fields": ["id", "name", "description", "traffic", "dist", "@deprecated arch", "lang", "cpu", "cpu_benchmark", "memory_size", "hdd_size", "hdd_text", "hdd_count", "datacenter", "network_speed"],
        "addon_product_fields": ["id", "name", "type", "price"],
    }:
        fail("creation-response policy changed")
    if fixture.get("local_policy") != {
        "cost": "catalog-gross-recurring-plus-setup-scale-4",
        "account": "required-credential-bound-authorization-evidence",
        "catalog_observation": "authenticated-execution-stable-credential",
        "transaction_observation": "authenticated-execution-stable-credential",
        "permit_minting": "one-shot-strong-digest",
        "market_created_addons": "must-be-empty",
        "addon_created_price": "exact-catalog-price",
        "addon_reconciliation_identity": "server-number-plus-product-id-conservative",
        "addon_get_type": "optional-documented-example",
        "addon_created_type": "required-documented-schema",
        "response_bytes": 1048576,
        "automatic_retry": "never",
        "single_attempt": "no-idempotency-key",
        "uncertain_repeat": "fresh-exact-plan-plus-same-credential-plus-same-idempotency-plus-absent-transaction-multiset-proof",
        "ci_purchase": False,
    }:
        fail("local billable-operation policy changed")


def validate_implementation(module: Path) -> None:
    source = "\n".join(path.read_text(encoding="ascii") for path in module.rglob("*.rs"))
    authorization = (module / "authorization.rs").read_text(encoding="ascii")
    observation = (module.parent / "observation.rs").read_text(encoding="ascii")
    catalog_exchange = (module.parent / "exchange.rs").read_text(encoding="ascii")
    transaction_exchange = (module.parent / "transaction/exchange.rs").read_text(encoding="ascii")
    plan = (module.parent / "plan.rs").read_text(encoding="ascii")
    prepare = (module / "prepare.rs").read_text(encoding="ascii")
    permit = (module / "permit.rs").read_text(encoding="ascii")
    reconcile = (module / "reconcile.rs").read_text(encoding="ascii")
    request = (module / "request.rs").read_text(encoding="ascii")
    market_created = (module.parent / "transaction/decode/market_created.rs").read_text(encoding="ascii")
    addon_decode = (module.parent / "transaction/decode/addon.rs").read_text(encoding="ascii")
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
        "BoundCredentialTransport",
        "build_plan_digest_with_authorization_evidence",
        "permit_minted.replace(true)",
        "AuthorityAlreadyMinted",
        "CredentialMismatch",
        "PermitValidity",
        "ReplayPolicy",
        "RobotOrderNotApplied",
        "reconcile_not_applied",
        "core::ptr::eq",
    ]:
        if token not in permit:
            fail(f"cost authority lost {token}")
    for token in [
        "RobotOrderAccount",
        "RobotOrderAuthorizationEvidence",
        "RobotOrderPermitRequest",
        "for_request",
        "request.credential_binding()",
        "PlanAuthorizationEvidence",
        "CredentialBinding",
    ]:
        if token not in authorization:
            fail(f"authorization evidence lost {token}")
    normalized_permit = permit.replace("\n", "").replace(" ", "")
    if (
        "credential_matches=request.credential_binding().matches(authorization.credential())"
        not in normalized_permit
        or "if!credential_matches" not in normalized_permit
    ):
        fail("confirmation authorization/request credential check changed")
    for token in ["CredentialObserved", "from_parts", "credential", "[redacted]"]:
        if token not in observation:
            fail(f"credential observation lost {token}")
    for exchange in [catalog_exchange, transaction_exchange]:
        for token in [
            "execute_observed_blocking",
            "execute_observed_async",
            "execute_observed_local_async",
            "require_stable_credential",
            "CredentialMismatch",
        ]:
            if token not in exchange:
                fail(f"authenticated observation execution lost {token}")
        if exchange.count("execute_observed_blocking") != 3:
            fail("blocking observation execution coverage changed")
    for token in ["CredentialObserved", "CredentialMismatch", ".credential().matches"]:
        if token not in plan:
            fail(f"catalog credential association lost {token}")
    if permit.count("core::ptr::eq") != 2:
        fail("request/proof identity checks changed")
    if permit.count("BoundCredentialTransport") != 5:
        fail("credential-bound dispatch coverage changed")
    for token in [
        "MatchingTransaction",
        ".any(|value| self.matches_transaction(value))",
        ".any(|value| self.matches_reconciliation_transaction(value))",
        "matches_reconciliation_transaction",
        ".filter(|candidate| *candidate == selection.addon().id())",
        "transactions.credential()",
        "value.addons().is_empty()",
        "price_matches",
        "pair_matches",
    ]:
        if token not in reconcile:
            fail(f"reconciliation lost {token}")
    if reconcile.count("MatchingTransaction") != 5:
        fail("matching-transaction rejection coverage changed")
    if reconcile.count("price_matches") != 2:
        fail("addon exact-price comparison coverage changed")
    normalized_reconcile = reconcile.replace("\n", "").replace(" ", "")
    if (
        "fnmatches_reconciliation_transaction(&self,value:&RobotAddonTransaction)->bool{"
        "value.server_number()==self.plan.server()"
        "&&value.product().id()==self.plan.product().id()}"
        not in normalized_reconcile
    ):
        fail("addon reconciliation identity is no longer conservative")
    if (
        "self.matches_reconciliation_transaction(value)"
        "&&value.product().kind().is_some_and"
        not in normalized_reconcile
        or "&&price_matches(self.plan.product().price(),value.product().price())"
        not in normalized_reconcile
    ):
        fail("addon creation response lost strict type or price validation")
    if "impl PrepareOperation for" in source or "pub fn as_untyped" in source:
        fail("billable request regained an unguarded or type-erased execution route")
    for token in ["RobotRipeReason", "ip_ipv4", "subnet_ipv4", "failover_subnet_ipv4"]:
        if token not in request:
            fail(f"addon parameter policy lost {token}")
    for token in ["addons", "network_speed"]:
        if token not in market_created:
            fail(f"market creation decoder lost {token}")
    for token in [
        "RequiredForCreation",
        "OptionalForDocumentedGet",
        "decode_addon_created",
        'require_fields(product, &["id", "name", "type", "price"])',
    ]:
        if token not in addon_decode:
            fail(f"addon type policy lost {token}")
    if addon_decode.count("RequiredForCreation") != 3:
        fail("creation-required addon type coverage changed")


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
