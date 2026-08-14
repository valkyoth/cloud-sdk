#!/usr/bin/env python3
"""Mutation tests for the Robot firewall contract checker."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
CHECK = ROOT / "scripts/check_robot_firewalls.py"
FIXTURE = ROOT / "tests/fixtures/robot-firewall/v0.89.0.json"
API_LOCK = ROOT / "tests/fixtures/robot-api/v0.74.0.json"
FUZZ_HARNESS = ROOT / "scripts/check_fuzz_harness.sh"


def run(
    fixture: Path,
    api_lock: Path = API_LOCK,
    fuzz_harness: Path = FUZZ_HARNESS,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            "python3",
            str(CHECK),
            "--fixture",
            str(fixture),
            "--api-lock",
            str(api_lock),
            "--fuzz-harness",
            str(fuzz_harness),
        ],
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
    changed["operations"][4]["success"] = 200
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["request"]["replacement_intents"] = ["inline-rules"]
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["local_policy"]["rules_per_direction"] = 101
    mutations.append(changed)
    with tempfile.TemporaryDirectory() as directory:
        for index, mutation in enumerate(mutations):
            path = Path(directory) / f"mutation-{index}.json"
            path.write_text(json.dumps(mutation), encoding="ascii")
            assert run(path).returncode != 0
        harness = Path(directory) / "check_fuzz_harness.sh"
        source = FUZZ_HARNESS.read_text(encoding="ascii")
        reviewed = (
            'elif [ "$target" = robot_firewall_response ]; then\n'
            "            # One selector byte plus the complete 2 MiB list-response boundary.\n"
            "            max_len=2097153"
        )
        assert reviewed in source
        harness.write_text(
            source.replace(reviewed, reviewed.replace("2097153", "262145")),
            encoding="ascii",
        )
        assert run(FIXTURE, fuzz_harness=harness).returncode != 0
    print("5 Robot firewall contract regression groups passed.")


if __name__ == "__main__":
    main()
