#!/usr/bin/env python3
"""Mutation tests for the Robot vSwitch contract checker."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
CHECK = ROOT / "scripts/check_robot_vswitches.py"
FIXTURE = ROOT / "tests/fixtures/robot-vswitch/v0.90.0.json"
API_LOCK = ROOT / "tests/fixtures/robot-api/v0.74.0.json"
FUZZ_HARNESS = ROOT / "scripts/check_fuzz_harness.sh"
FORM_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/vswitch/form.rs"
PREPARE_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/vswitch/prepare.rs"
DECODE_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/vswitch/decode.rs"
EXCHANGE_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/vswitch/exchange.rs"
PERMIT_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/vswitch/permit.rs"


def run(
    fixture: Path = FIXTURE,
    api_lock: Path = API_LOCK,
    fuzz_harness: Path = FUZZ_HARNESS,
    form_source: Path = FORM_SOURCE,
    prepare_source: Path = PREPARE_SOURCE,
    decode_source: Path = DECODE_SOURCE,
    exchange_source: Path = EXCHANGE_SOURCE,
    permit_source: Path = PERMIT_SOURCE,
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
            "--prepare-source",
            str(prepare_source),
            "--decode-source",
            str(decode_source),
            "--exchange-source",
            str(exchange_source),
            "--permit-source",
            str(permit_source),
        ],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )


def main() -> None:
    assert run().returncode == 0, run()
    original = json.loads(FIXTURE.read_text(encoding="ascii"))
    mutations = []
    changed = json.loads(json.dumps(original))
    changed["operations"][1]["success"] = 200
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["operations"][0]["quota"]["requests"] = 100
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["request"]["membership_field"] = "server"
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["response"]["server_statuses"] = ["ready", "failed"]
    mutations.append(changed)
    changed = json.loads(json.dumps(original))
    changed["local_policy"]["membership_request_items"] = 4096
    mutations.append(changed)

    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        for index, mutation in enumerate(mutations):
            path = root / f"mutation-{index}.json"
            path.write_text(json.dumps(mutation), encoding="ascii")
            assert run(fixture=path).returncode != 0
        assert_mutated_source_fails(root, "form.rs", FORM_SOURCE, "server[]", "server")
        assert_mutated_source_fails(
            root,
            "prepare.rs",
            PREPARE_SOURCE,
            "ResponseBodyPolicy::Forbidden",
            "ResponseBodyPolicy::Required",
            prepare_source=True,
        )
        assert_mutated_source_fails(
            root,
            "decode.rs",
            DECODE_SOURCE,
            "valid_route(network, prefix, gateway)",
            "true",
            decode_source=True,
        )
        assert_mutated_source_fails(
            root,
            "exchange.rs",
            EXCHANGE_SOURCE,
            "subsequent `GET /vswitch/{id}`",
            "unspecified follow-up",
            exchange_source=True,
        )
        assert_mutated_source_fails(
            root,
            "permit.rs",
            PERMIT_SOURCE,
            "RobotVSwitchCancelRequest",
            "MissingCancelRequest",
            permit_source=True,
        )
        harness = root / "check_fuzz_harness.sh"
        harness_source = FUZZ_HARNESS.read_text(encoding="ascii")
        reviewed_fuzz = (
            'elif [ "$target" = robot_vswitch_response ]; then\n'
            "            # One selector byte plus the complete 1 MiB response boundary.\n"
            "            max_len=1048577"
        )
        assert reviewed_fuzz in harness_source
        harness.write_text(
            harness_source.replace(
                reviewed_fuzz, reviewed_fuzz.replace("1048577", "1048576")
            ),
            encoding="ascii",
        )
        assert run(fuzz_harness=harness).returncode != 0
    print("11 Robot vSwitch contract regression groups passed.")


def assert_mutated_source_fails(
    root: Path,
    name: str,
    source: Path,
    old: str,
    new: str,
    *,
    prepare_source: bool = False,
    decode_source: bool = False,
    exchange_source: bool = False,
    permit_source: bool = False,
) -> None:
    path = root / name
    text = source.read_text(encoding="ascii")
    assert old in text
    path.write_text(text.replace(old, new, 1), encoding="ascii")
    kwargs = {"form_source": path}
    if prepare_source:
        kwargs = {"prepare_source": path}
    elif decode_source:
        kwargs = {"decode_source": path}
    elif exchange_source:
        kwargs = {"exchange_source": path}
    elif permit_source:
        kwargs = {"permit_source": path}
    assert run(**kwargs).returncode != 0


if __name__ == "__main__":
    main()
