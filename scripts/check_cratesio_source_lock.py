#!/usr/bin/env python3
"""Validate committed crates.io source evidence and optionally re-fetch it."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from cratesio_source_fetch import fetch_source
from cratesio_source_lock import (
    CARGO_COLUMNS,
    TSV_COLUMNS,
    SourceLockError,
    observe,
    validate_lock,
    validate_tsv,
)


ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "provider-drift" / "providers" / "cratesio-source.lock.json"
OPERATIONS = ROOT / "docs" / "CRATESIO_API_SCOPE.tsv"
CARGO = ROOT / "docs" / "CRATESIO_CARGO_COMPATIBILITY.tsv"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--fetch",
        action="store_true",
        help="fetch official sources and compare a rebuilt observation",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        lock = json.loads(LOCK.read_text(encoding="ascii"))
        validate_lock(lock)
        operation_lock = OPERATIONS.read_bytes()
        cargo_lock = CARGO.read_bytes()
        operation_rows = validate_tsv(
            operation_lock, TSV_COLUMNS, lock["openapi"]["operations"]
        )
        cargo_rows = validate_tsv(
            cargo_lock,
            CARGO_COLUMNS,
            lock["cargo"]["stable_operations"] + lock["cargo"]["instruction_targets"],
        )
        if sum(row["stability"] == "stable-cargo" for row in operation_rows) != 7:
            raise SourceLockError("accepted stable Cargo operation count changed")
        if sum(row["classification"] == "superseded" for row in cargo_rows) != 7:
            raise SourceLockError("accepted Cargo overlap classification changed")
        if args.fetch:
            payloads = {source["id"]: fetch_source(source) for source in lock["sources"]}
            operations, cargo, summary = observe(lock, payloads)
            if operations != operation_lock:
                raise SourceLockError("live OpenAPI observation differs from accepted scope")
            if cargo != cargo_lock:
                raise SourceLockError("live Cargo contract differs from accepted compatibility lock")
            if summary["openapi"] != lock["openapi"] or summary["cargo"] != lock["cargo"]:
                raise SourceLockError("live crates.io source summary changed")
            if summary["policy"] != lock["policy"]:
                raise SourceLockError("live crates.io access policy changed")
    except (OSError, UnicodeError, json.JSONDecodeError, SourceLockError) as error:
        print(f"crates.io source lock: {error}", file=sys.stderr)
        return 1
    mode = "live sources" if args.fetch else "committed evidence"
    print(
        "crates.io source lock: "
        f"{lock['openapi']['operations']} operations, "
        f"{lock['cargo']['stable_operations']} Cargo overlaps, and {mode} passed."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
