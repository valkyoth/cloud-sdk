#!/usr/bin/env python3
"""Bind OVHcloud cursor and schema headers to reviewed probe evidence."""

from __future__ import annotations

import csv
import sys
from pathlib import Path

from provider_drift_model import read_bounded_json, validate_lock


ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "provider-drift/providers/ovhcloud-v2-probe.lock.json"
FIXTURE = ROOT / "crates/cloud-sdk/tests/fixtures/ovhcloud-header-contracts.tsv"
FIELDS = (
    "cursor_request",
    "size_request",
    "next_response",
    "terminal",
    "page_size",
    "operation_count",
    "schema_header",
    "schema_version",
    "schema_major",
    "principles_sha256",
    "schema_sha256",
)


def fixture_row() -> dict[str, str]:
    try:
        with FIXTURE.open("r", encoding="ascii", newline="") as handle:
            reader = csv.DictReader(handle, delimiter="\t")
            if tuple(reader.fieldnames or ()) != FIELDS:
                raise ValueError("header fixture fields are invalid")
            rows = list(reader)
    except (OSError, UnicodeError, csv.Error) as error:
        raise ValueError("header fixture could not be read") from error
    if len(rows) != 1:
        raise ValueError("header fixture must contain exactly one row")
    return rows[0]


def one_contract(lock: dict, category: str, contract_id: str) -> dict:
    matches = [
        row["values"]
        for row in lock["contracts"][category]
        if row.get("id") == contract_id
    ]
    if len(matches) != 1:
        raise ValueError(f"{contract_id} contract is ambiguous")
    return matches[0]


def check() -> None:
    lock = validate_lock(read_bounded_json(LOCK, "OVHcloud probe lock"))
    fixture = fixture_row()
    cursor = one_contract(lock, "pagination", "iam-cursor")
    headers = one_contract(lock, "headers", "cursor-pagination")
    schema = one_contract(lock, "headers", "schema-version-validation")
    expected = {
        "cursor_request": cursor["cursor_request"],
        "size_request": cursor["size_request"],
        "next_response": cursor["next_response"],
        "terminal": cursor["terminal"],
        "page_size": "5",
        "operation_count": str(cursor["operation_count"]),
        "schema_header": schema["request"],
        "schema_version": schema["reviewed_version"],
        "schema_major": str(schema["reviewed_major"]),
        "principles_sha256": schema["source_sha256"],
        "schema_sha256": schema["schema_source_sha256"],
    }
    if fixture != expected:
        raise ValueError("header fixture differs from source-locked contracts")
    if headers.get("request") != [cursor["cursor_request"], cursor["size_request"]]:
        raise ValueError("cursor request headers differ across source contracts")
    if headers.get("response") != [cursor["next_response"]]:
        raise ValueError("cursor response header differs across source contracts")
    paginated = [
        row for row in lock["contracts"]["operations"] if row["values"]["headers"]
    ]
    if len(paginated) != cursor["operation_count"]:
        raise ValueError("paginated operation count differs from source evidence")
    if schema.get("use") != "validation_only" or not schema.get(
        "account_default_when_absent"
    ):
        raise ValueError("schema header is not validation-only")


def main() -> int:
    try:
        check()
    except (KeyError, OSError, TypeError, UnicodeError, ValueError) as error:
        print(f"OVHcloud header conformance: {error}", file=sys.stderr)
        return 1
    print("OVHcloud cursor and schema headers are source-bound.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
