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
MODULE = ROOT / "crates/cloud-sdk-hetzner/src/robot/ordering/mutation"


def run(**overrides: Path) -> subprocess.CompletedProcess[str]:
    values = {"fixture": FIXTURE, "api_lock": API_LOCK, "module": MODULE}
    values.update(overrides)
    command = ["python3", str(CHECK)]
    for name, path in values.items():
        command.extend([f"--{name.replace('_', '-')}", str(path)])
    return subprocess.run(command, cwd=ROOT, check=False, capture_output=True, text=True)


def copy_module(destination: Path, old: str, new: str) -> Path:
    shutil.copytree(MODULE, destination)
    for path in destination.rglob("*.rs"):
        text = path.read_text(encoding="ascii")
        if old in text:
            path.write_text(text.replace(old, new, 1), encoding="ascii")
            return destination
    raise AssertionError(f"mutation token not found: {old}")


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
        ]:
            changed = json.loads(json.dumps(original))
            mutate(changed)
            mutations.append(changed)
        for index, changed in enumerate(mutations):
            path = root / f"fixture-{index}.json"
            path.write_text(json.dumps(changed), encoding="ascii")
            assert run(fixture=path).returncode != 0
        cases = [
            ("RetryEligibility::Never", "RetryEligibility::ExplicitPolicy"),
            ("CostIntent::MayIncurCost", "CostIntent::NoKnownCost"),
            ("Some(request.plan_cost())", "None"),
            ("core::ptr::eq", "core::ptr::addr_eq"),
            ("MatchingTransaction", "TransactionIgnored"),
        ]
        for index, (old, new) in enumerate(cases):
            module = copy_module(root / f"module-{index}", old, new)
            result = run(module=module)
            assert result.returncode != 0, (old, result)
    print("9 Robot billable-order regression groups passed.")


if __name__ == "__main__":
    main()
