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
THREAT_MODEL = ROOT / "docs/THREAT_MODEL_DELTA_0.92.0.md"
MIGRATION = ROOT / "docs/MIGRATION_0.92.0.md"
RELEASE_NOTES = ROOT / "release-notes/RELEASE_NOTES_0.92.0.md"
WIRE_LOCK = ROOT / "docs/ROBOT_WIRE_SOURCE_LOCK.md"
PUBLIC_API = ROOT / "docs/PUBLIC_API_REVIEW_0.92.0.md"
SPEC_LOCK = ROOT / "docs/SPEC_LOCK.md"
FUZZ_MANIFEST = ROOT / "fuzz/Cargo.toml"
FUZZ_GATE = ROOT / "scripts/check_fuzz_harness.sh"
FUZZ_SOURCE = ROOT / "fuzz/fuzz_targets/robot_transaction_response.rs"
FUZZ_SEEDS = ROOT / "fuzz/seeds/robot_transaction_response"

FUZZ_SEED_HASHES = {
    "official-addon-list.json": "1102fc9f1cc50878cf72b100791996ffc5df0262bdceda6bd6bcf6c83c4d7677",
    "official-market-list.json": "f9515aba25ed66bd58f618eed43e18b6cce09f263c725c2172efe7bb5df9a900",
    "official-standard-list.json": "a60c780548dab11eecd77d539326aeb18a942f07a55d35c3f979aa75e0964b15",
    "valid-addon-detail.json": "7f419e43cbfe58a934c24cfbcad5644995b9bfc9a00ae464bec409f9b688cc6e",
    "valid-market-detail.json": "f1e750d7ae2ddf6e1c2583f6af24adee3c01da2d5eacedc6ea2554c8eb4782ba",
    "valid-standard-detail.json": "56a9524c4bce30abe75cc350b6027ecd743d106b444f05b17559cc8bef70af37",
}

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
    threat_model_path: Path,
    migration_path: Path,
    release_notes_path: Path,
    wire_lock_path: Path,
    public_api_path: Path,
    spec_lock_path: Path,
    fuzz_manifest_path: Path,
    fuzz_gate_path: Path,
    fuzz_source_path: Path,
    fuzz_seeds_path: Path,
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
    validate_sources(
        module_path,
        readme_path,
        threat_model_path,
        migration_path,
        release_notes_path,
        wire_lock_path,
        public_api_path,
        spec_lock_path,
        fuzz_manifest_path,
        fuzz_gate_path,
        fuzz_source_path,
        fuzz_seeds_path,
    )


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
    module: Path,
    readme: Path,
    threat_model: Path,
    migration: Path,
    release_notes: Path,
    wire_lock: Path,
    public_api: Path,
    spec_lock: Path,
    fuzz_manifest: Path,
    fuzz_gate: Path,
    fuzz_source: Path,
    fuzz_seeds: Path,
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
    if "unreachable!" in prepare:
        fail("transaction preparation retains a panic-only invariant path")
    if "impl PrepareOperation for" in nested:
        fail("transaction requests regained a raw-storage preparation route")
    for token in [
        ".map_err(RobotOrderRequestError::InvalidTarget)?",
        ".map_err(RobotOrderRequestError::InvalidPreparedPolicy)?",
        "admitted!(validate_target(target_storage, target_len));",
        "rust-lang/rust#54663",
        "must prepare through `PreparationStorageGuard`",
        "pub fn prepare_guarded<'guard>",
        "storage: &'guard mut PreparationStorageGuard<'_>",
        "storage.prepare_with(|buffers| prepare($kind, buffers))",
    ]:
        if token not in prepare:
            fail(f"typed transaction preparation failure lost {token}")
    exchange = files.get("exchange.rs", "")
    for token in [
        "storage: &'guard mut PreparationStorageGuard<'_>",
        "self.prepare_guarded(storage)?",
    ]:
        if token not in exchange:
            fail(f"guarded response association lost {token}")
    module_doc = module.with_suffix(".rs").read_text(encoding="ascii")
    for token in ["```compile_fail", "request.prepare(PreparationStorage::new"]:
        if token not in module_doc:
            fail(f"raw-storage compile-fail evidence lost {token}")
    request = files.get("request.rs", "")
    for token in [
        "ROBOT_ORDER_TRANSACTION_QUOTA",
        "max_requests: 500",
        "interval: DelaySeconds::new(3_600)",
        "This is one account-level budget",
    ]:
        if token not in request:
            fail(f"shared transaction quota evidence lost {token}")
    for token in ["RobotOrderTransactionStatus::Ready", "valid_rfc3339", "reject_transaction_duplicates", "reject_duplicates_by_cmp"]:
        if token not in nested:
            fail(f"strict transaction evidence lost {token}")
    if files.get("exchange.rs", "").count("ResponseIdentityMismatch") != 3:
        fail("detail response identity binding changed")
    if "status == 404 && code == \"NOT_FOUND\"" not in files.get("failure.rs", ""):
        fail("source-locked NOT_FOUND decoder changed")
    readme_text = readme.read_text(encoding="ascii")
    if "all six active read-only transaction operations" not in readme_text:
        fail("provider README lost transaction scope")
    if "Transaction preparation requires `PreparationStorageGuard` directly" not in readme_text:
        fail("provider README lost guarded transaction cleanup boundary")
    documentation_tokens = {
        threat_model: [
            "every reachable validation and encoding failure",
            "rust-lang/rust#54663",
            "exposes no raw `PreparationStorage` preparation route",
            "Unsafe lifetime emulation was rejected",
        ],
        migration: [
            "requires `&mut PreparationStorageGuard` directly",
            "do not implement raw-storage `PrepareOperation`",
        ],
        release_notes: [
            "Reachable failures clear both buffers before target binding",
            "require `&mut PreparationStorageGuard`",
            "no raw-storage `PrepareOperation` route",
        ],
        wire_lock: [
            "require `PreparationStorageGuard` directly",
            "no raw `PreparationStorage` preparation route",
        ],
        public_api: [
            "expose `prepare_guarded`",
            "do not implement raw-storage `PrepareOperation`",
        ],
        spec_lock: [
            "requires `PreparationStorageGuard` directly",
            "does not implement raw-storage `PrepareOperation`",
        ],
    }
    for path, tokens in documentation_tokens.items():
        content = " ".join(path.read_text(encoding="ascii").split())
        for token in tokens:
            if token not in content:
                fail(f"cleanup boundary evidence lost {token} in {path.name}")
    manifest = fuzz_manifest.read_text(encoding="ascii")
    gate = fuzz_gate.read_text(encoding="ascii")
    if 'name = "robot_transaction_response"' not in manifest:
        fail("transaction response fuzz target is missing")
    if "passed for 34 targets" not in gate or "max_len=4194304" not in gate:
        fail("transaction fuzz boundary is not source-locked")
    source = fuzz_source.read_text(encoding="ascii")
    if "split_first" in source:
        fail("transaction fuzzing still depends on a shallow selector prefix")
    for request_type in [
        "RobotStandardTransactionListRequest::new()",
        'RobotStandardTransactionGetRequest::new(id("B-fuzz"))',
        "RobotMarketTransactionListRequest::new()",
        'RobotMarketTransactionGetRequest::new(id("B-fuzz"))',
        "RobotAddonTransactionListRequest::new()",
        'RobotAddonTransactionGetRequest::new(id("B-fuzz"))',
    ]:
        if source.count(request_type) != 1:
            fail(f"deep fuzz decoder coverage changed for {request_type}")
    validate_fuzz_seeds(fuzz_seeds)


def validate_fuzz_seeds(directory: Path) -> None:
    for name, expected_hash in FUZZ_SEED_HASHES.items():
        path = directory / name
        try:
            content = path.read_bytes()
            value = json.loads(content)
        except (OSError, UnicodeError, json.JSONDecodeError) as error:
            fail(f"cannot read deep fuzz seed {name}: {error}")
        if hashlib.sha256(content).hexdigest() != expected_hash:
            fail(f"deep fuzz seed changed: {name}")
        if name.startswith("official-"):
            if not isinstance(value, list) or len(value) != 2:
                fail(f"deep list fuzz seed lost provider examples: {name}")
        else:
            if not isinstance(value, dict):
                fail(f"deep detail fuzz seed is not an object: {name}")
            transaction = value.get("transaction")
            if not isinstance(transaction, dict) or transaction.get("id") != "B-fuzz":
                fail(f"deep detail fuzz seed lost request identity: {name}")
    standard = load_seed(directory / "valid-standard-detail.json")
    market = load_seed(directory / "valid-market-detail.json")
    addon = load_seed(directory / "valid-addon-detail.json")
    if not standard["transaction"].get("authorized_key"):
        fail("standard detail fuzz seed lost key parsing depth")
    if "cpu_benchmark" not in market["transaction"].get("product", {}):
        fail("market detail fuzz seed lost hardware parsing depth")
    if not addon["transaction"].get("resources"):
        fail("addon detail fuzz seed lost resource parsing depth")


def load_seed(path: Path) -> dict:
    value = json.loads(path.read_text(encoding="ascii"))
    if not isinstance(value, dict):
        fail(f"deep detail fuzz seed is not an object: {path.name}")
    return value


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture", type=Path, default=FIXTURE)
    parser.add_argument("--api-lock", type=Path, default=API_LOCK)
    parser.add_argument("--module", type=Path, default=MODULE)
    parser.add_argument("--readme", type=Path, default=README)
    parser.add_argument("--threat-model", type=Path, default=THREAT_MODEL)
    parser.add_argument("--migration", type=Path, default=MIGRATION)
    parser.add_argument("--release-notes", type=Path, default=RELEASE_NOTES)
    parser.add_argument("--wire-lock", type=Path, default=WIRE_LOCK)
    parser.add_argument("--public-api", type=Path, default=PUBLIC_API)
    parser.add_argument("--spec-lock", type=Path, default=SPEC_LOCK)
    parser.add_argument("--fuzz-manifest", type=Path, default=FUZZ_MANIFEST)
    parser.add_argument("--fuzz-gate", type=Path, default=FUZZ_GATE)
    parser.add_argument("--fuzz-source", type=Path, default=FUZZ_SOURCE)
    parser.add_argument("--fuzz-seeds", type=Path, default=FUZZ_SEEDS)
    args = parser.parse_args()
    validate(
        args.fixture,
        args.api_lock,
        args.module,
        args.readme,
        args.threat_model,
        args.migration,
        args.release_notes,
        args.wire_lock,
        args.public_api,
        args.spec_lock,
        args.fuzz_manifest,
        args.fuzz_gate,
        args.fuzz_source,
        args.fuzz_seeds,
    )
    print("Robot transaction source contract passed.")


if __name__ == "__main__":
    main()
