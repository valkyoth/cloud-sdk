#!/usr/bin/env python3
"""Mutation tests for the Robot traffic contract checker."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
CHECK = ROOT / "scripts/check_robot_traffic.py"
FIXTURE = ROOT / "tests/fixtures/robot-traffic/v0.87.0.json"
API_LOCK = ROOT / "tests/fixtures/robot-api/v0.74.0.json"


def run(fixture: Path, api_lock: Path = API_LOCK) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["python3", str(CHECK), "--fixture", str(fixture), "--api-lock", str(api_lock)],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )


def main() -> None:
    assert run(FIXTURE).returncode == 0
    original = json.loads(FIXTURE.read_text(encoding="ascii"))
    mutations = []
    changed = json.loads(json.dumps(original))
    changed["operation"]["quota"]["requests"] = 201
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["request"]["types"] = ["day", "month"]
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["local_limits"]["single_value_targets"] = 251
    mutations.append(changed)
    with tempfile.TemporaryDirectory() as directory:
        for index, mutation in enumerate(mutations):
            path = Path(directory) / f"mutation-{index}.json"
            path.write_text(json.dumps(mutation), encoding="ascii")
            assert run(path).returncode != 0
    print("4 Robot traffic contract regression groups passed.")


if __name__ == "__main__":
    main()
