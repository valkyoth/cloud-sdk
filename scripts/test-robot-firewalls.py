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
FORM_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/firewall/form.rs"
RECONCILE_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/firewall/reconcile.rs"
NAME_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/firewall/types.rs"


def run(
    fixture: Path,
    api_lock: Path = API_LOCK,
    fuzz_harness: Path = FUZZ_HARNESS,
    form_source: Path = FORM_SOURCE,
    reconcile_source: Path = RECONCILE_SOURCE,
    name_source: Path = NAME_SOURCE,
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
            "--form-source",
            str(form_source),
            "--reconcile-source",
            str(reconcile_source),
            "--name-source",
            str(name_source),
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
    changed = json.loads(json.dumps(original))
    changed["operations"][1]["quota"]["requests"] = 200
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["request"]["port_protocol_policy"] = "protocol-required-with-port"
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["response"]["template_optional_fields"] = []
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
        form = Path(directory) / "form.rs"
        form.write_text(
            FORM_SOURCE.read_text(encoding="ascii").replace(
                "let mut text = String::new();", "let mut text = String::from(\"\");"
            ),
            encoding="ascii",
        )
        assert run(FIXTURE, form_source=form).returncode != 0
        reconcile = Path(directory) / "reconcile.rs"
        reconcile.write_text(
            RECONCILE_SOURCE.read_text(encoding="ascii").replace(
                "constant_time_eq", "ordinary_eq"
            ),
            encoding="ascii",
        )
        assert run(FIXTURE, reconcile_source=reconcile).returncode != 0
        names = Path(directory) / "types.rs"
        names.write_text(
            NAME_SOURCE.read_text(encoding="ascii").replace("\\u{061c}", "\\u{0061}"),
            encoding="ascii",
        )
        assert run(FIXTURE, name_source=names).returncode != 0
    print("11 Robot firewall contract regression groups passed.")


if __name__ == "__main__":
    main()
