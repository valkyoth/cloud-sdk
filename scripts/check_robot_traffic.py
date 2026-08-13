#!/usr/bin/env python3
"""Validate the immutable Robot traffic source contract."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests/fixtures/robot-traffic/v0.87.0.json"
API_LOCK = ROOT / "tests/fixtures/robot-api/v0.74.0.json"
MAX_BYTES = 64 * 1024


def fail(message: str) -> None:
    raise SystemExit(f"Robot traffic contract: {message}")


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
    return {
        "schema_version": 1,
        "source": {
            "retrieved": "2026-08-13",
            "url": "https://robot.hetzner.com/doc/webservice/en.html",
            "sha256": "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a",
        },
        "inventory": "tests/fixtures/robot-api/v0.74.0.json",
        "operation": {
            "id": "robot_get_traffic",
            "inventory_id": "get_traffic",
            "method": "POST",
            "path": "/traffic",
            "impact": "read-only",
            "quota": {"requests": 200, "interval_seconds": 3600},
        },
        "request": {
            "content_type": "application/x-www-form-urlencoded",
            "required": ["target", "from", "to", "type"],
            "repeated_targets": ["ip[]", "subnet[]"],
            "types": ["day", "month", "year"],
            "single_values": "optional-true",
        },
        "response": {
            "status": 200,
            "content_type": "application/json",
            "fields": ["traffic.type", "traffic.from", "traffic.to", "traffic.data"],
            "amount_fields": ["in", "out", "sum"],
            "units": "GB",
            "missing_targets": "omitted",
            "single_value_keys": {"day": "00-23", "month": "01-31", "year": "01-12"},
        },
        "failures": {
            "400": ["INVALID_INPUT"],
            "404": ["NOT_FOUND"],
            "500": ["INTERNAL_ERROR"],
        },
        "local_limits": {
            "response_bytes": 8_388_608,
            "targets": 4_092,
            "single_value_targets": 250,
            "number_bytes": 128,
            "object_fields": 4_096,
        },
    }


def validate(fixture: Path, api_lock: Path) -> None:
    if read(fixture) != expected():
        fail("fixture differs from the reviewed v0.87 contract")
    operations = read(api_lock).get("operations")
    if not isinstance(operations, list):
        fail("API lock operations are missing")
    matches = [
        item for item in operations
        if isinstance(item, dict)
        and item.get("id") == "get_traffic"
        and item.get("method") == "POST"
        and item.get("path") == "/traffic"
        and item.get("status") == "active"
        and item.get("milestone") == "v0.87.0"
    ]
    if len(matches) != 1:
        fail("API inventory does not contain the exact active traffic row")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture", type=Path, default=FIXTURE)
    parser.add_argument("--api-lock", type=Path, default=API_LOCK)
    args = parser.parse_args()
    validate(args.fixture, args.api_lock)
    print("Robot traffic source contract passed.")


if __name__ == "__main__":
    main()
