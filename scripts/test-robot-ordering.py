#!/usr/bin/env python3
"""Mutation tests for the Robot ordering-catalog contract checker."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
CHECK = ROOT / "scripts/check_robot_ordering.py"
FIXTURE = ROOT / "tests/fixtures/robot-ordering/v0.91.0.json"
API_LOCK = ROOT / "tests/fixtures/robot-api/v0.74.0.json"
PREPARE = ROOT / "crates/cloud-sdk-hetzner/src/robot/ordering/prepare.rs"
DECODE = ROOT / "crates/cloud-sdk-hetzner/src/robot/ordering/decode"
EXCHANGE = ROOT / "crates/cloud-sdk-hetzner/src/robot/ordering/exchange.rs"
PLAN = ROOT / "crates/cloud-sdk-hetzner/src/robot/ordering/plan.rs"
FUZZ_HARNESS = ROOT / "scripts/check_fuzz_harness.sh"


def run(**overrides: Path) -> subprocess.CompletedProcess[str]:
    values = {
        "fixture": FIXTURE,
        "api_lock": API_LOCK,
        "prepare": PREPARE,
        "decode": DECODE,
        "exchange": EXCHANGE,
        "plan": PLAN,
        "fuzz_harness": FUZZ_HARNESS,
    }
    values.update(overrides)
    command = ["python3", str(CHECK)]
    for name, path in values.items():
        command.extend([f"--{name.replace('_', '-')}", str(path)])
    return subprocess.run(command, cwd=ROOT, check=False, capture_output=True, text=True)


def main() -> None:
    baseline = run()
    assert baseline.returncode == 0, baseline
    original = json.loads(FIXTURE.read_text(encoding="ascii"))
    mutations = []
    changed = json.loads(json.dumps(original))
    changed["operations"][0]["method"] = "POST"
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["operations"][5]["quota"]["requests"] = 499
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["local_policy"]["decimal_fractional_digits"] = 5
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["local_policy"]["plans_are_executable"] = True
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["request"]["authentication"] = "bearer"
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["response"]["market_fields"].remove("next_reduce_date")
    mutations.append(changed)

    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        for index, mutation in enumerate(mutations):
            path = root / f"fixture-{index}.json"
            path.write_text(json.dumps(mutation), encoding="ascii")
            assert run(fixture=path).returncode != 0
        mutate_source(root, "prepare.rs", PREPARE, "OperationImpact::ReadOnly", "OperationImpact::Mutation", "prepare")
        mutate_source(root, "plan.rs", PLAN, "RevalidateImmediatelyBeforePurchase", "PriceUnchecked", "plan")
        harness = root / "check_fuzz_harness.sh"
        source = FUZZ_HARNESS.read_text(encoding="ascii")
        assert "max_len=4194305" in source
        harness.write_text(source.replace("max_len=4194305", "max_len=1048577"), encoding="ascii")
        assert run(fuzz_harness=harness).returncode != 0
    print("9 Robot ordering catalog regression groups passed.")


def mutate_source(
    root: Path,
    name: str,
    source_path: Path,
    old: str,
    new: str,
    argument: str,
) -> None:
    source = source_path.read_text(encoding="ascii")
    assert old in source
    path = root / name
    path.write_text(source.replace(old, new), encoding="ascii")
    assert run(**{argument: path}).returncode != 0


if __name__ == "__main__":
    main()
