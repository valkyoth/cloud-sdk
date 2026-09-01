#!/usr/bin/env python3
"""Stage a complete crates.io lock refresh without changing accepted evidence."""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import sys
import tempfile
from pathlib import Path
from typing import Any

from cratesio_drift_adapter import build_observation
from cratesio_drift_documents import provider_lock, source_projection
from cratesio_source_fetch import observe_source
from cratesio_source_lock import CARGO_COLUMNS, TSV_COLUMNS, observe, validate_tsv
from cratesio_source_manifest import (
    SOURCE_DETAILS,
    official_urls,
    validate_artifact_digests,
    validate_lock as validate_source_lock,
)
from provider_drift_model import (
    CATEGORIES,
    MAX_DOCUMENT_BYTES,
    ModelError,
    canonical_bytes,
    read_bounded_json,
    validate_lock,
    validate_observation,
)
from provider_drift_report import build_report


BUNDLE_FORMAT = "cloud-sdk-cratesio-refresh-candidate/v1"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    stage = subparsers.add_parser("stage")
    stage.add_argument("--source-commit", required=True)
    stage.add_argument("--reviewed-at", required=True)
    stage.add_argument("--output", required=True, type=Path)
    verify = subparsers.add_parser("verify")
    verify.add_argument("candidate", type=Path)
    return parser.parse_args()


def source_template(source_commit: str, reviewed_at: str) -> dict[str, Any]:
    if re.fullmatch(r"[0-9a-f]{40}", source_commit) is None:
        raise ModelError("source commit must be lowercase Git SHA-1")
    if re.fullmatch(r"\d{4}-\d{2}-\d{2}", reviewed_at) is None:
        raise ModelError("review date must use YYYY-MM-DD")
    urls = official_urls(source_commit)
    sources = []
    for identity in sorted(SOURCE_DETAILS):
        accept, media_type, maximum = SOURCE_DETAILS[identity]
        sources.append(
            {
                "id": identity,
                "url": urls[identity],
                "accept": accept,
                "final_url": urls[identity],
                "redirects": [],
                "media_type": media_type,
                "max_bytes": maximum,
                "size_bytes": 1,
                "sha256": "0" * 64,
            }
        )
    return {
        "format": 1,
        "reviewed_at": reviewed_at,
        "source_commit": source_commit,
        "sources": sources,
        "artifacts": {
            "operations_sha256": "0" * 64,
            "cargo_compatibility_sha256": "0" * 64,
        },
        "openapi": {},
        "cargo": {},
        "policy": {},
    }


def build_candidate(source_commit: str, reviewed_at: str) -> dict[str, Any]:
    source_lock = source_template(source_commit, reviewed_at)
    payloads = {
        source["id"]: observe_source(source) for source in source_lock["sources"]
    }
    for source in source_lock["sources"]:
        payload = payloads[source["id"]]
        source["size_bytes"] = len(payload)
        source["sha256"] = hashlib.sha256(payload).hexdigest()
    operations, cargo, summary = observe(source_lock, payloads)
    source_lock["artifacts"] = {
        "operations_sha256": hashlib.sha256(operations).hexdigest(),
        "cargo_compatibility_sha256": hashlib.sha256(cargo).hexdigest(),
    }
    source_lock.update(summary)
    empty = {category: [] for category in CATEGORIES}
    adapter_lock = provider_lock(source_lock, empty)
    observation = build_observation(adapter_lock, payloads)
    lock = provider_lock(source_lock, observation["contracts"])
    candidate = {
        "cargo_tsv": cargo.decode("ascii"),
        "format": BUNDLE_FORMAT,
        "observation": observation,
        "operations_tsv": operations.decode("ascii"),
        "provider_lock": lock,
        "source_lock": source_lock,
    }
    validate_candidate(candidate)
    return candidate


def validate_candidate(candidate: dict[str, Any]) -> dict[str, Any]:
    expected = {
        "cargo_tsv",
        "format",
        "observation",
        "operations_tsv",
        "provider_lock",
        "source_lock",
    }
    if set(candidate) != expected or candidate.get("format") != BUNDLE_FORMAT:
        raise ModelError("crates.io refresh candidate is incomplete")
    source_lock = candidate["source_lock"]
    validate_source_lock(source_lock)
    try:
        operations = candidate["operations_tsv"].encode("ascii")
        cargo = candidate["cargo_tsv"].encode("ascii")
    except (AttributeError, UnicodeError) as error:
        raise ModelError("candidate inventories are not ASCII text") from error
    validate_artifact_digests(source_lock, operations, cargo)
    validate_tsv(operations, TSV_COLUMNS, source_lock["openapi"]["operations"])
    validate_tsv(
        cargo,
        CARGO_COLUMNS,
        source_lock["cargo"]["stable_operations"]
        + source_lock["cargo"]["instruction_targets"],
    )
    lock = validate_lock(candidate["provider_lock"])
    observation = validate_observation(candidate["observation"])
    projected = source_projection(source_lock)
    if lock["sources"] != projected or observation["sources"] != projected:
        raise ModelError("candidate sources are not bound to the source manifest")
    if lock != provider_lock(source_lock, observation["contracts"]):
        raise ModelError("candidate provider lock differs from the reviewed policy")
    if build_report(lock, observation)["result"] != "clean":
        raise ModelError("candidate provider observation differs from its lock")
    return candidate


def write_once(path: Path, payload: bytes) -> None:
    if len(payload) > MAX_DOCUMENT_BYTES:
        raise ModelError("crates.io refresh candidate exceeds its hard bound")
    parent = path.parent.resolve(strict=True)
    descriptor, temporary = tempfile.mkstemp(prefix=".cratesio-refresh-", dir=parent)
    try:
        os.fchmod(descriptor, 0o600)
        with os.fdopen(descriptor, "wb", closefd=True) as output:
            output.write(payload)
            output.flush()
            os.fsync(output.fileno())
        os.link(temporary, path)
        directory = os.open(parent, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    finally:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass


def main() -> int:
    args = parse_args()
    try:
        if args.command == "stage":
            candidate = build_candidate(args.source_commit, args.reviewed_at)
            write_once(args.output, canonical_bytes(candidate) + b"\n")
            print(f"staged crates.io refresh candidate at {args.output}")
        else:
            validate_candidate(
                read_bounded_json(args.candidate, "crates.io refresh candidate")
            )
            print("crates.io refresh candidate is complete and internally clean")
    except (ModelError, OSError, TypeError, ValueError) as error:
        print(f"crates.io refresh: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
