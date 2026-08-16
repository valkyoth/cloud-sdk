#!/usr/bin/env python3
"""Mutation tests for the v0.93 Robot order contract checker."""

from __future__ import annotations

import json
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
CHECK = ROOT / "scripts/check_robot_order_mutations.py"
FIXTURE = ROOT / "tests/fixtures/robot-order-mutations/v0.93.0.json"
API_LOCK = ROOT / "tests/fixtures/robot-api/v0.74.0.json"
ORDERING = ROOT / "crates/cloud-sdk-hetzner/src/robot/ordering"
MODULE = ORDERING / "mutation"


def run(**overrides: Path) -> subprocess.CompletedProcess[str]:
    values = {"fixture": FIXTURE, "api_lock": API_LOCK, "module": MODULE}
    values.update(overrides)
    command = ["python3", str(CHECK)]
    for name, path in values.items():
        command.extend([f"--{name.replace('_', '-')}", str(path)])
    return subprocess.run(command, cwd=ROOT, check=False, capture_output=True, text=True)


def copy_ordering(destination: Path, relative: str, old: str, new: str) -> Path:
    shutil.copytree(ORDERING, destination)
    path = destination / relative
    text = path.read_text(encoding="ascii")
    if old not in text:
        raise AssertionError(f"mutation token not found in {relative}: {old}")
    path.write_text(text.replace(old, new, 1), encoding="ascii")
    return destination / "mutation"


def main() -> None:
    baseline = run()
    assert baseline.returncode == 0, baseline
    original = json.loads(FIXTURE.read_text(encoding="ascii"))
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        mutations = []
        for mutate in [
            lambda value: value["operations"][0].update(method="GET"),
            lambda value: value["operations"][2]["errors"].pop(1),
            lambda value: value["local_policy"].update(ci_purchase=True),
            lambda value: value["local_policy"].update(automatic_retry="automatic"),
            lambda value: value["local_policy"].update(permit_minting="unbounded"),
            lambda value: value["local_policy"].update(catalog_observation="caller-associated"),
            lambda value: value["local_policy"].update(addon_created_price="unchecked"),
            lambda value: value["responses"]["addon_product_fields"].remove("type"),
        ]:
            changed = json.loads(json.dumps(original))
            mutate(changed)
            mutations.append(changed)
        for index, changed in enumerate(mutations):
            path = root / f"fixture-{index}.json"
            path.write_text(json.dumps(changed), encoding="ascii")
            assert run(fixture=path).returncode != 0
        cases = [
            ("mutation/prepare.rs", "RetryEligibility::Never", "RetryEligibility::ExplicitPolicy"),
            ("mutation/prepare.rs", "CostIntent::MayIncurCost", "CostIntent::NoKnownCost"),
            ("mutation/permit.rs", "Some(request.plan_cost())", "None"),
            ("mutation/permit.rs", "core::ptr::eq", "core::ptr::addr_eq"),
            ("mutation/reconcile.rs", "MatchingTransaction", "TransactionIgnored"),
            ("mutation/permit.rs", "BoundCredentialTransport", "UnboundCredentialTransport"),
            ("mutation/permit.rs", "permit_minted.replace(true)", "permit_minted.get()"),
            ("exchange.rs", "execute_observed_blocking", "execute_unobserved_blocking"),
            ("mutation/authorization.rs", "request.credential_binding()", "CredentialBinding::default()"),
            ("mutation/permit.rs", ".matches(authorization.credential())", ".matches(request.credential_binding())"),
            ("mutation/reconcile.rs", "value.addons().is_empty()", "true"),
            ("mutation/reconcile.rs", "price_matches", "price_ignored"),
            (
                "mutation/reconcile.rs",
                "matches_reconciliation_transaction",
                "matches_permissive_transaction",
            ),
            ("transaction/decode/addon.rs", "RequiredForCreation", "OptionalForDocumentedGet"),
        ]
        for index, (relative, old, new) in enumerate(cases):
            module = copy_ordering(root / f"ordering-{index}", relative, old, new)
            result = run(module=module)
            assert result.returncode != 0, (old, result)
    print("23 Robot billable-order regression groups passed.")


if __name__ == "__main__":
    main()
