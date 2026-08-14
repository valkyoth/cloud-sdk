#!/usr/bin/env python3
"""Validate the immutable Robot vSwitch source contract."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests/fixtures/robot-vswitch/v0.90.0.json"
API_LOCK = ROOT / "tests/fixtures/robot-api/v0.74.0.json"
FUZZ_HARNESS = ROOT / "scripts/check_fuzz_harness.sh"
FORM_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/vswitch/form.rs"
PREPARE_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/vswitch/prepare.rs"
DECODE_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/vswitch/decode.rs"
EXCHANGE_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/vswitch/exchange.rs"
PERMIT_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/vswitch/permit.rs"
MAX_BYTES = 64 * 1024
FIXTURE_SHA256 = "b16f953a609659505bf181485eed7c317d555ebe5d48cf0388448ec67b1fc971"


def fail(message: str) -> None:
    raise SystemExit(f"Robot vSwitch contract: {message}")


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
    prepare_source_path: Path,
    decode_source_path: Path,
    exchange_source_path: Path,
    permit_source_path: Path,
) -> None:
    try:
        fixture_payload = fixture.read_bytes()
    except OSError as error:
        fail(f"could not read {fixture}: {error}")
    digest = hashlib.sha256(fixture_payload).hexdigest()
    if fixture == FIXTURE and digest != FIXTURE_SHA256:
        fail(f"fixture digest changed to {digest}")
    reviewed = read(FIXTURE)
    if read(fixture) != reviewed:
        fail("fixture differs from the reviewed v0.90 contract")

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
        and item.get("group") == "vswitch"
        and item.get("status") == "active"
        and item.get("milestone") == "v0.90.0"
    }
    if actual_rows != expected_rows or len(expected_rows) != 7:
        fail("API inventory does not contain the exact seven active vSwitch rows")

    quotas = [item["quota"]["requests"] for item in reviewed["operations"]]
    if quotas != [500, 100, 500, 100, 100, 100, 100]:
        fail("source-locked vSwitch quotas changed")
    if any(item["quota"]["interval_seconds"] != 3600 for item in reviewed["operations"]):
        fail("source-locked vSwitch quota interval changed")
    if [item["body"] for item in reviewed["operations"]] != [
        "json-list",
        "json-detail",
        "json-detail",
        "empty",
        "empty",
        "empty",
        "empty",
    ]:
        fail("success-body policy changed")

    request = reviewed["request"]
    if request.get("membership_field") != "server[]":
        fail("repeated membership field changed")
    if request.get("membership_order") != "caller-order-repeated-fields":
        fail("membership ordering contract changed")
    response = reviewed["response"]
    if response.get("server_statuses") != ["ready", "in process", "failed"]:
        fail("server membership statuses changed")
    if len(response.get("empty_acknowledgements", [])) != 4:
        fail("empty acknowledgement inventory changed")
    policy = reviewed["local_policy"]
    expected_policy = {
        "vlan_range": [1, 4094],
        "name_bytes": 128,
        "membership_request_items": 256,
        "list_items": 4096,
        "member_servers": 4096,
        "subnets": 4096,
        "cloud_networks": 4096,
        "list_response_bytes": 1048576,
        "item_response_bytes": 1048576,
        "automatic_mutation_retry": False,
        "empty_mutation_requires_get_reconciliation": True,
    }
    if policy != expected_policy:
        fail("local vSwitch safety policy changed")

    expected_examples = {
        "tests/fixtures/robot-vswitch/official-detail-response.json": (
            "568931bcf1d640889d76b97b36e21b0a372f7a41f1e2e113c034c0cda6b5918e"
        ),
        "tests/fixtures/robot-vswitch/official-list-response.json": (
            "ccb7722cd06f8ccc43e6ae23ead66e928bf025b16891df837653032be4b2807b"
        ),
    }
    examples = reviewed["source"].get("examples")
    if not isinstance(examples, list) or {
        item.get("path"): item.get("sha256")
        for item in examples
        if isinstance(item, dict)
    } != expected_examples:
        fail("official example inventory changed")
    for relative, expected_digest in expected_examples.items():
        try:
            payload = (ROOT / relative).read_bytes()
        except OSError as error:
            fail(f"could not read {relative}: {error}")
        if hashlib.sha256(payload).hexdigest() != expected_digest:
            fail(f"official example digest changed for {relative}")

    harness = read_text(fuzz_harness)
    fuzz_limit = re.search(
        r'elif \[ "\$target" = robot_vswitch_response \]; then\n'
        r'(?:[^\n]*\n)*?\s*max_len=([0-9]+)\n',
        harness,
    )
    if fuzz_limit is None or fuzz_limit.group(1) != "1048577":
        fail("vSwitch fuzzing no longer admits the full response boundary")

    form_source = read_text(form_source_path)
    for required in [
        'field("server[]", selector.as_str())',
        "try_reserve_exact(selectors.len())",
        "RobotFormField::sensitive",
        'field("cancellation_date", "now")',
    ]:
        if required not in form_source:
            fail(f"vSwitch form contract lost {required}")
    prepare_source = read_text(prepare_source_path)
    for operation in reviewed["operations"]:
        if f'"{operation["id"]}"' not in prepare_source:
            fail(f"prepared operation ID missing: {operation['id']}")
    for required in ["ContentTypePolicy::Forbidden", "ResponseBodyPolicy::Forbidden"]:
        if required not in prepare_source:
            fail(f"empty response policy lost {required}")
    decode_source = read_text(decode_source_path)
    for required in [
        "require_fields(",
        "reject_duplicates_by_cmp",
        "valid_route(network, prefix, gateway)",
        "MAX_ROBOT_VSWITCH_MEMBER_SERVERS",
    ]:
        if required not in decode_source:
            fail(f"strict vSwitch decoding lost {required}")
    exchange_source = read_text(exchange_source_path)
    if "subsequent `GET /vswitch/{id}`" not in exchange_source:
        fail("empty mutation reconciliation warning is missing")
    permit_source = read_text(permit_source_path)
    for request_type in [
        "RobotVSwitchCreateRequest",
        "RobotVSwitchUpdateRequest",
        "RobotVSwitchCancelRequest",
        "RobotVSwitchAddServersRequest",
        "RobotVSwitchRemoveServersRequest",
    ]:
        if request_type not in permit_source:
            fail(f"permit coverage lost {request_type}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture", type=Path, default=FIXTURE)
    parser.add_argument("--api-lock", type=Path, default=API_LOCK)
    parser.add_argument("--fuzz-harness", type=Path, default=FUZZ_HARNESS)
    parser.add_argument("--form-source", type=Path, default=FORM_SOURCE)
    parser.add_argument("--prepare-source", type=Path, default=PREPARE_SOURCE)
    parser.add_argument("--decode-source", type=Path, default=DECODE_SOURCE)
    parser.add_argument("--exchange-source", type=Path, default=EXCHANGE_SOURCE)
    parser.add_argument("--permit-source", type=Path, default=PERMIT_SOURCE)
    args = parser.parse_args()
    validate(
        args.fixture,
        args.api_lock,
        args.fuzz_harness,
        args.form_source,
        args.prepare_source,
        args.decode_source,
        args.exchange_source,
        args.permit_source,
    )
    print("Robot vSwitch source contract passed.")


if __name__ == "__main__":
    main()
