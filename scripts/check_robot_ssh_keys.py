#!/usr/bin/env python3
"""Validate the immutable Robot SSH-key source contract."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests/fixtures/robot-ssh-keys/v0.88.0.json"
API_LOCK = ROOT / "tests/fixtures/robot-api/v0.74.0.json"
FUZZ_HARNESS = ROOT / "scripts/check_fuzz_harness.sh"
MAX_BYTES = 64 * 1024
FIXTURE_SHA256 = "913a447077962f5d1993ff21af7f4699cc4be0b2472d6fbc4bd6c37fb793db4e"


def fail(message: str) -> None:
    raise SystemExit(f"Robot SSH-key contract: {message}")


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


def expected() -> dict[str, Any]:
    return json.loads(
        (ROOT / "tests/fixtures/robot-ssh-keys/v0.88.0.json").read_text(
            encoding="ascii"
        )
    )


def validate(fixture: Path, api_lock: Path, fuzz_harness: Path) -> None:
    try:
        fixture_digest = hashlib.sha256(fixture.read_bytes()).hexdigest()
    except OSError as error:
        fail(f"could not hash {fixture}: {error}")
    if fixture_digest != FIXTURE_SHA256:
        fail(f"fixture digest changed to {fixture_digest}")
    reviewed = expected()
    if read(fixture) != reviewed:
        fail("fixture differs from the reviewed v0.88 contract")
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
        and item.get("group") == "ssh_keys"
        and item.get("status") == "active"
        and item.get("milestone") == "v0.88.0"
    }
    if actual_rows != expected_rows:
        fail("API inventory does not contain the exact five active SSH-key rows")
    try:
        harness = fuzz_harness.read_text(encoding="ascii")
    except (OSError, UnicodeError) as error:
        fail(f"could not read {fuzz_harness}: {error}")
    fuzz_limit = re.search(
        r'elif \[ "\$target" = robot_ssh_key_response \]; then\n'
        r'(?:[^\n]*\n)*?\s*max_len=([0-9]+)\n',
        harness,
    )
    if fuzz_limit is None or fuzz_limit.group(1) != "2097153":
        fail("fuzzing must admit one selector plus the complete 2 MiB list response")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture", type=Path, default=FIXTURE)
    parser.add_argument("--api-lock", type=Path, default=API_LOCK)
    parser.add_argument("--fuzz-harness", type=Path, default=FUZZ_HARNESS)
    args = parser.parse_args()
    validate(args.fixture, args.api_lock, args.fuzz_harness)
    print("Robot SSH-key source contract passed.")


if __name__ == "__main__":
    main()
