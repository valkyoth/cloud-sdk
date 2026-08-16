#!/usr/bin/env python3
"""Mutation tests for the Robot transaction contract checker."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
CHECK = ROOT / "scripts/check_robot_transactions.py"
FIXTURE = ROOT / "tests/fixtures/robot-transactions/v0.92.0.json"
API_LOCK = ROOT / "tests/fixtures/robot-api/v0.74.0.json"
MODULE = ROOT / "crates/cloud-sdk-hetzner/src/robot/ordering/transaction"
README = ROOT / "crates/cloud-sdk-hetzner/README.md"
FUZZ_MANIFEST = ROOT / "fuzz/Cargo.toml"
FUZZ_GATE = ROOT / "scripts/check_fuzz_harness.sh"
FUZZ_SOURCE = ROOT / "fuzz/fuzz_targets/robot_transaction_response.rs"
FUZZ_SEEDS = ROOT / "fuzz/seeds/robot_transaction_response"


def run(**overrides: Path) -> subprocess.CompletedProcess[str]:
    values = {
        "fixture": FIXTURE,
        "api_lock": API_LOCK,
        "module": MODULE,
        "readme": README,
        "fuzz_manifest": FUZZ_MANIFEST,
        "fuzz_gate": FUZZ_GATE,
        "fuzz_source": FUZZ_SOURCE,
        "fuzz_seeds": FUZZ_SEEDS,
    }
    values.update(overrides)
    command = ["python3", str(CHECK)]
    for name, path in values.items():
        command.extend([f"--{name.replace('_', '-')}", str(path)])
    return subprocess.run(command, cwd=ROOT, check=False, capture_output=True, text=True)


def mutate_source(root: Path, relative: str, old: str, new: str) -> Path:
    destination = root / "module"
    for source in MODULE.rglob("*.rs"):
        target = destination / source.relative_to(MODULE)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(source.read_text(encoding="ascii"), encoding="ascii")
    path = destination / relative
    source = path.read_text(encoding="ascii")
    assert old in source
    path.write_text(source.replace(old, new), encoding="ascii")
    return destination


def copy_seeds(destination: Path) -> Path:
    destination.mkdir(parents=True, exist_ok=True)
    for source in FUZZ_SEEDS.iterdir():
        if source.is_file():
            (destination / source.name).write_bytes(source.read_bytes())
    return destination


def main() -> None:
    baseline = run()
    assert baseline.returncode == 0, baseline
    original = json.loads(FIXTURE.read_text(encoding="ascii"))
    mutations = []
    changed = json.loads(json.dumps(original))
    changed["operations"][0]["method"] = "POST"
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["operations"][1]["window_days"] = 30
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["response"]["statuses"].append("failed")
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["local_policy"]["detail_identity"] = "unbound"
    mutations.append(changed)

    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        for index, mutation in enumerate(mutations):
            path = root / f"fixture-{index}.json"
            path.write_text(json.dumps(mutation), encoding="ascii")
            assert run(fixture=path).returncode != 0
        module = mutate_source(root / "prepare", "prepare.rs", "OperationImpact::ReadOnly", "OperationImpact::Mutation")
        assert run(module=module).returncode != 0
        module = mutate_source(root / "quota", "request.rs", "max_requests: 500", "max_requests: 501")
        assert run(module=module).returncode != 0
        module = mutate_source(
            root / "panic",
            "prepare.rs",
            ".map_err(RobotOrderRequestError::InvalidTarget)?",
            ".unwrap_or_else(|_| unreachable!())",
        )
        assert run(module=module).returncode != 0
        module = mutate_source(root / "identity", "exchange.rs", "ResponseIdentityMismatch", "IdentityIgnored")
        assert run(module=module).returncode != 0
        module = mutate_source(root / "failure", "failure.rs", 'status == 404 && code == "NOT_FOUND"', 'status == 400 && code == "INVALID_INPUT"')
        assert run(module=module).returncode != 0
        gate = root / "fuzz-gate.sh"
        gate.write_text(FUZZ_GATE.read_text(encoding="ascii").replace("passed for 34 targets", "passed for 33 targets"), encoding="ascii")
        assert run(fuzz_gate=gate).returncode != 0
        fuzz_source = root / "fuzz-source.rs"
        fuzz_source.write_text(
            FUZZ_SOURCE.read_text(encoding="ascii").replace(
                "RobotAddonTransactionGetRequest::new(id(\"B-fuzz\"))",
                "RobotAddonTransactionListRequest::new()",
            ),
            encoding="ascii",
        )
        assert run(fuzz_source=fuzz_source).returncode != 0
        seeds = copy_seeds(root / "seeds")
        detail = seeds / "valid-addon-detail.json"
        detail.write_bytes(detail.read_bytes() + b" ")
        assert run(fuzz_seeds=seeds).returncode != 0
    print("12 Robot transaction regression groups passed.")


if __name__ == "__main__":
    main()
