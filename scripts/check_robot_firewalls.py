#!/usr/bin/env python3
"""Validate the immutable Robot firewall source contract."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests/fixtures/robot-firewall/v0.89.0.json"
API_LOCK = ROOT / "tests/fixtures/robot-api/v0.74.0.json"
FUZZ_HARNESS = ROOT / "scripts/check_fuzz_harness.sh"
FORM_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/firewall/form.rs"
RECONCILE_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/firewall/reconcile.rs"
NAME_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/firewall/types.rs"
MAX_BYTES = 64 * 1024
FIXTURE_SHA256 = "238cec5b8cc51546483cae4336eedc65d644eb27e69179b5f70173702abc8538"


def fail(message: str) -> None:
    raise SystemExit(f"Robot firewall contract: {message}")


def read(path: Path) -> dict[str, Any]:
    try:
        payload = path.read_bytes()
    except OSError as error:
        fail(f"could not read {path}: {error}")
    if len(payload) > MAX_BYTES:
        fail(f"{path} exceeds 64 KiB")
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"{path} is not valid UTF-8 JSON: {error}")
    if not isinstance(value, dict):
        fail(f"{path} root is not an object")
    return value


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="ascii")
    except (OSError, UnicodeError) as error:
        fail(f"could not read {path}: {error}")


def validate(
    fixture: Path,
    api_lock: Path,
    fuzz_harness: Path,
    form_source_path: Path,
    reconcile_source_path: Path,
    name_source_path: Path,
) -> None:
    try:
        fixture_payload = fixture.read_bytes()
    except OSError as error:
        fail(f"could not read {fixture}: {error}")
    digest = hashlib.sha256(fixture_payload).hexdigest()
    if digest != FIXTURE_SHA256:
        fail(f"fixture digest changed to {digest}")
    reviewed = read(FIXTURE)
    if read(fixture) != reviewed:
        fail("fixture differs from the reviewed v0.89 contract")
    operations = read(api_lock).get("operations")
    if not isinstance(operations, list):
        fail("API lock operations are missing")
    expected_rows = {
        (item["inventory_id"], item["method"], item["path"])
        for item in reviewed["operations"]
    }
    actual_rows = {
        (item.get("id"), item.get("method"), item.get("path"))
        for item in operations
        if isinstance(item, dict)
        and item.get("group") == "firewall"
        and item.get("status") == "active"
        and item.get("milestone") == "v0.89.0"
    }
    if actual_rows != expected_rows:
        fail("API inventory does not contain the exact eight active firewall rows")
    expected_quota = {"requests": 500, "interval_seconds": 3600}
    if len(reviewed["operations"]) != 8:
        fail("source contract must contain exactly eight operations")
    for operation in reviewed["operations"]:
        if operation.get("quota") != expected_quota:
            fail(f"{operation.get('id', '[missing]')} has incorrect source quota")
    if reviewed["request"]["replacement_intents"] != ["inline-rules", "template-id"]:
        fail("replacement intents no longer preserve the source conflict")
    if reviewed["request"].get("port_protocol_policy") != "protocol-may-be-omitted-with-port":
        fail("missing-protocol port rules are no longer source locked")
    response = reviewed["response"]
    if response.get("template_required_fields") != [
        "id",
        "filter_ipv6",
        "whitelist_hos",
        "is_default",
        "rules",
    ] or response.get("template_optional_fields") != ["name"]:
        fail("detailed template name optionality changed")
    if response.get("template_name_source_conflict") != (
        "output-table-required-but-official-detailed-examples-omit-name"
    ):
        fail("template-name source contradiction is not recorded")
    expected_examples = {
        "tests/fixtures/robot-firewall/official-firewall-response.json": (
            "9801694f6488c98cf2f4a340f12ad2ccb74d2439423f5e8650b98c00c714092d"
        ),
        "tests/fixtures/robot-firewall/official-template-response.json": (
            "9df462ec80cc1dc8e7a669d113b04f331b8ea00ecfa2feaadf89e929b71adec2"
        ),
    }
    examples = reviewed["source"].get("examples")
    if not isinstance(examples, list) or {
        item.get("path"): item.get("sha256")
        for item in examples
        if isinstance(item, dict)
    } != expected_examples:
        fail("official example inventory changed")
    decoded_examples = {}
    for relative, expected_digest in expected_examples.items():
        path = ROOT / relative
        try:
            payload = path.read_bytes()
        except OSError as error:
            fail(f"could not read {path}: {error}")
        if hashlib.sha256(payload).hexdigest() != expected_digest:
            fail(f"official example digest changed for {relative}")
        decoded_examples[relative] = read(path)
    try:
        source_rule = decoded_examples[next(iter(expected_examples))]["firewall"]["rules"][
            "input"
        ][0]
        source_template = decoded_examples[
            "tests/fixtures/robot-firewall/official-template-response.json"
        ]["firewall_template"]
    except (KeyError, IndexError, TypeError):
        fail("official examples no longer have the reviewed shape")
    if source_rule.get("dst_port") != "80" or source_rule.get("protocol") is not None:
        fail("official missing-protocol port example changed")
    if "name" in source_template:
        fail("official detailed template example unexpectedly contains name")
    if reviewed["local_policy"]["rules_per_direction"] != 100:
        fail("per-direction rule limit changed")
    try:
        harness = fuzz_harness.read_text(encoding="ascii")
    except (OSError, UnicodeError) as error:
        fail(f"could not read {fuzz_harness}: {error}")
    fuzz_limit = re.search(
        r'elif \[ "\$target" = robot_firewall_response \]; then\n'
        r'(?:[^\n]*\n)*?\s*max_len=([0-9]+)\n',
        harness,
    )
    if fuzz_limit is None or fuzz_limit.group(1) != "2097153":
        fail("fuzzing must admit one selector plus the complete 2 MiB list response")
    form_source = read_text(form_source_path)
    if "format!(" in form_source or "String::from" in form_source:
        fail("firewall form allocation bypass returned")
    if form_source.count("try_reserve_exact") < 3:
        fail("firewall form allocations are no longer explicitly fallible")
    reconcile_source = read_text(reconcile_source_path)
    if "constant_time_eq" not in reconcile_source or ".fold(" not in reconcile_source:
        fail("protected firewall reconciliation is no longer fixed-work")
    name_source = read_text(name_source_path)
    for scalar in ["061c", "200b", "200f", "202a", "202e", "2060", "2069", "feff"]:
        if f"\\u{{{scalar}}}" not in name_source:
            fail(f"prohibited name scalar U+{scalar.upper()} is missing")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture", type=Path, default=FIXTURE)
    parser.add_argument("--api-lock", type=Path, default=API_LOCK)
    parser.add_argument("--fuzz-harness", type=Path, default=FUZZ_HARNESS)
    parser.add_argument("--form-source", type=Path, default=FORM_SOURCE)
    parser.add_argument("--reconcile-source", type=Path, default=RECONCILE_SOURCE)
    parser.add_argument("--name-source", type=Path, default=NAME_SOURCE)
    args = parser.parse_args()
    validate(
        args.fixture,
        args.api_lock,
        args.fuzz_harness,
        args.form_source,
        args.reconcile_source,
        args.name_source,
    )
    print("Robot firewall source contract passed.")


if __name__ == "__main__":
    main()
