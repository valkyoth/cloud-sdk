#!/usr/bin/env python3
"""Validate immutable v0.91.0 Robot ordering-catalog evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests/fixtures/robot-ordering/v0.91.0.json"
API_LOCK = ROOT / "tests/fixtures/robot-api/v0.74.0.json"
PREPARE = ROOT / "crates/cloud-sdk-hetzner/src/robot/ordering/prepare.rs"
DECODE = ROOT / "crates/cloud-sdk-hetzner/src/robot/ordering/decode"
EXCHANGE = ROOT / "crates/cloud-sdk-hetzner/src/robot/ordering/exchange.rs"
PLAN = ROOT / "crates/cloud-sdk-hetzner/src/robot/ordering/plan.rs"
VALUE = ROOT / "crates/cloud-sdk-hetzner/src/robot/ordering/value.rs"
README = ROOT / "crates/cloud-sdk-hetzner/README.md"
FUZZ_MANIFEST = ROOT / "fuzz/Cargo.toml"
FUZZ_HARNESS = ROOT / "scripts/check_fuzz_harness.sh"

OPERATIONS = [
    ("robot_list_server_products", "list_server_products", "/order/server/product", "json-list"),
    ("robot_get_server_product", "get_server_product", "/order/server/product/{product-id}", "json-detail"),
    ("robot_list_server_market_products", "list_server_market_products", "/order/server_market/product", "json-list"),
    ("robot_get_server_market_product", "get_server_market_product", "/order/server_market/product/{product-id}", "json-detail"),
    ("robot_list_server_addon_products", "list_server_addon_products", "/order/server_addon/{server-number}/product", "json-list"),
    ("robot_list_order_currencies", "list_order_currencies", "/order/currency", "json-detail"),
]


def fail(message: str) -> None:
    raise SystemExit(f"Robot ordering contract: {message}")


def load(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="ascii"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        fail(f"cannot read {path}: {error}")


def validate(
    fixture_path: Path,
    api_lock_path: Path,
    prepare_path: Path,
    decode_path: Path,
    exchange_path: Path,
    plan_path: Path,
    value_path: Path,
    readme_path: Path,
    fuzz_manifest_path: Path,
    fuzz_harness_path: Path,
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

    operations = fixture.get("operations")
    expected = [
        {
            "id": operation_id,
            "inventory_id": inventory_id,
            "method": "GET",
            "path": path,
            "success": 200,
            "quota": {"requests": 500, "interval_seconds": 3600},
            "body": body,
        }
        for operation_id, inventory_id, path, body in OPERATIONS
    ]
    if operations != expected:
        fail("six-operation catalog contract changed")
    validate_inventory(api_lock_path)
    validate_schema(fixture)

    prepare = prepare_path.read_text(encoding="ascii")
    for operation_id, _, path, _ in OPERATIONS:
        if operation_id not in prepare or path.split("{")[0].rstrip("/") not in prepare:
            fail(f"implementation lost {operation_id}")
    for token in [
        "OperationImpact::ReadOnly",
        "RequestSemantics::Safe",
        "RetryEligibility::ExplicitPolicy",
        "CostIntent::NoKnownCost",
        "MAX_ROBOT_ORDER_LIST_RESPONSE_BYTES: usize = 4_194_304",
    ]:
        if token not in prepare:
            fail(f"preparation policy lost {token}")
    if "Method::Post" in prepare or "Method::Delete" in prepare:
        fail("ordering catalog exposes a mutation method")

    decode = "\n".join(
        path.read_text(encoding="ascii") for path in sorted(decode_path.glob("*.rs"))
    )
    for token in [
        "RobotOrderDecimal::new",
        "validate_architectures",
        "reject_duplicates_by_cmp",
        "hourly_net.is_some() != hourly_gross.is_some()",
    ]:
        if token not in decode:
            fail(f"strict decoding evidence lost {token}")

    exchange = exchange_path.read_text(encoding="ascii")
    for token in [
        "ResponseIdentityMismatch",
        "pub struct RobotAddonCatalog<'request>",
        "request: &'request RobotAddonProductListRequest",
        "Result<RobotAddonCatalog<'request>",
    ]:
        if token not in exchange:
            fail(f"typed exchange lost request provenance: {token}")

    plan = plan_path.read_text(encoding="ascii")
    for token in [
        "RevalidateImmediatelyBeforePurchase",
        "LocationMismatch",
        "DuplicateAddon",
        "InvalidQuantity",
        "catalog: &'catalog RobotAddonCatalog<'request>",
        "RobotStandardAddonSelection([redacted])",
    ]:
        if token not in plan:
            fail(f"non-executable plan evidence lost {token}")
    if "impl PrepareOperation" in plan or "TransportRequest" in plan:
        fail("catalog plan became executable")
    if "server: &'a RobotServerNumber" in plan:
        fail("addon plan accepts a replaceable server identity")

    value = value_path.read_text(encoding="ascii")
    for token in [
        "sanitize_value(&mut self.coefficient);",
        "sanitize_value(&mut self.scale);",
    ]:
        if token not in value:
            fail(f"decimal scalar cleanup lost {token}")

    readme = readme_path.read_text(encoding="ascii")
    if "storage.prepare_with(|buffers| request.prepare_bound(buffers))?;" not in readme:
        fail("ordering example lost successful-path storage cleanup")

    manifest = fuzz_manifest_path.read_text(encoding="ascii")
    harness = fuzz_harness_path.read_text(encoding="ascii")
    if 'name = "robot_ordering_response"' not in manifest:
        fail("ordering response fuzz target is missing")
    if 'max_len=4194305' not in harness or "passed for 33 targets" not in harness:
        fail("ordering fuzz boundary is not source-locked")


def validate_examples(examples: object) -> None:
    if not isinstance(examples, list) or len(examples) != 4:
        fail("four reviewed response examples are required")
    for item in examples:
        if not isinstance(item, dict):
            fail("response example metadata is malformed")
        relative = item.get("path")
        expected = item.get("sha256")
        if not isinstance(relative, str) or not isinstance(expected, str):
            fail("response example metadata is incomplete")
        path = ROOT / relative
        try:
            actual = hashlib.sha256(path.read_bytes()).hexdigest()
        except OSError as error:
            fail(f"cannot read response example {relative}: {error}")
        if actual != expected:
            fail(f"response example digest changed: {relative}")


def validate_inventory(path: Path) -> None:
    inventory = load(path).get("operations", [])
    selected = {
        item.get("id"): item
        for item in inventory
        if item.get("group") == "ordering_catalog"
    }
    if set(selected) != {item[1] for item in OPERATIONS}:
        fail("API inventory ordering-catalog membership changed")
    for _, inventory_id, path_value, _ in OPERATIONS:
        item = selected[inventory_id]
        if item.get("method") != "GET" or item.get("path") != path_value:
            fail(f"API inventory drifted for {inventory_id}")
        if item.get("status") != "active" or item.get("milestone") != "v0.91.0":
            fail(f"API inventory status drifted for {inventory_id}")


def validate_schema(fixture: dict) -> None:
    request = fixture.get("request", {})
    expected_request = {
        "standard_filters": [
            "min_price",
            "max_price",
            "min_price_setup",
            "max_price_setup",
            "location",
        ],
        "content_type": None,
        "authentication": "robot-basic",
    }
    if request != expected_request:
        fail("catalog request schema changed")
    response = fixture.get("response", {})
    expected_response = {
        "content_type": "application/json",
        "standard_fields": [
            "id", "name", "description", "traffic", "dist",
            "@deprecated arch", "lang", "location", "prices",
            "orderable_addons",
        ],
        "market_fields": [
            "id", "name", "description", "traffic", "dist",
            "@deprecated arch", "lang", "cpu", "cpu_benchmark",
            "memory_size", "hdd_size", "hdd_text", "hdd_count",
            "datacenter", "network_speed", "price", "price_hourly",
            "price_setup", "price_vat", "price_hourly_vat",
            "price_setup_vat", "fixed_price", "next_reduce",
            "next_reduce_date", "orderable_addons",
        ],
        "addon_fields": ["id", "name", "type", "price"],
        "price_fields": ["location", "price", "price_setup"],
        "currency_fields": ["currency"],
    }
    if response != expected_response:
        fail("catalog response schema changed")
    policy = fixture.get("local_policy", {})
    expected = {
        "decimal_fractional_digits": 4,
        "decimal_total_digits": 18,
        "product_items": 4096,
        "nested_items": 4096,
        "list_response_bytes": 4194304,
        "item_response_bytes": 1048576,
        "plans_are_executable": False,
        "price_warning": "revalidate-immediately-before-purchase",
        "automatic_retry": "explicit-policy",
    }
    if policy != expected:
        fail("local catalog policy changed")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture", type=Path, default=FIXTURE)
    parser.add_argument("--api-lock", type=Path, default=API_LOCK)
    parser.add_argument("--prepare", type=Path, default=PREPARE)
    parser.add_argument("--decode", type=Path, default=DECODE)
    parser.add_argument("--exchange", type=Path, default=EXCHANGE)
    parser.add_argument("--plan", type=Path, default=PLAN)
    parser.add_argument("--value", type=Path, default=VALUE)
    parser.add_argument("--readme", type=Path, default=README)
    parser.add_argument("--fuzz-manifest", type=Path, default=FUZZ_MANIFEST)
    parser.add_argument("--fuzz-harness", type=Path, default=FUZZ_HARNESS)
    args = parser.parse_args()
    validate(
        args.fixture,
        args.api_lock,
        args.prepare,
        args.decode,
        args.exchange,
        args.plan,
        args.value,
        args.readme,
        args.fuzz_manifest,
        args.fuzz_harness,
    )
    print("Robot ordering catalog source contract passed.")


if __name__ == "__main__":
    main()
