#!/usr/bin/env python3
"""Stage a complete crates.io lock refresh without changing accepted evidence."""

from __future__ import annotations

import argparse
import base64
import binascii
import hashlib
import hmac
import json
import os
import re
import sys
import tempfile
from datetime import date
from pathlib import Path
from typing import Any

from cratesio_drift_adapter import build_observation, validate_stable_cargo_matches
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
    ModelError,
    canonical_bytes,
    read_bounded_bytes,
    validate_lock,
    validate_observation,
)
from provider_drift_report import build_report


BUNDLE_FORMAT = "cloud-sdk-cratesio-refresh-candidate/v2"
MAX_CANDIDATE_BYTES = 8 * 1024 * 1024


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
    try:
        parsed_reviewed_at = date.fromisoformat(reviewed_at)
    except ValueError as error:
        raise ModelError("review date must be a real YYYY-MM-DD date") from error
    if parsed_reviewed_at.isoformat() != reviewed_at:
        raise ModelError("review date is not canonical")
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
        "payloads": encode_payloads(payloads),
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
        "payloads",
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
    validate_stable_cargo_matches(observation)
    projected = source_projection(source_lock)
    if lock["sources"] != projected or observation["sources"] != projected:
        raise ModelError("candidate sources are not bound to the source manifest")
    if lock != provider_lock(source_lock, observation["contracts"]):
        raise ModelError("candidate provider lock differs from the reviewed policy")
    if build_report(lock, observation)["result"] != "clean":
        raise ModelError("candidate provider observation differs from its lock")
    verify_payload_binding(candidate)
    return candidate


def encode_payloads(payloads: dict[str, bytes]) -> dict[str, str]:
    return {
        identity: base64.b64encode(payload).decode("ascii")
        for identity, payload in sorted(payloads.items())
    }


def decode_candidate_payloads(
    encoded: Any, source_lock: dict[str, Any]
) -> dict[str, bytes]:
    sources = {source["id"]: source for source in source_lock["sources"]}
    if not isinstance(encoded, dict) or set(encoded) != set(sources):
        raise ModelError("candidate payload set differs from its source manifest")
    payloads = {}
    for identity, value in encoded.items():
        maximum = sources[identity]["max_bytes"]
        encoded_maximum = ((maximum + 2) // 3) * 4
        if not isinstance(value, str) or len(value) > encoded_maximum:
            raise ModelError(f"{identity} candidate payload exceeds its hard bound")
        try:
            payload = base64.b64decode(value.encode("ascii"), validate=True)
        except (UnicodeEncodeError, binascii.Error) as error:
            raise ModelError(f"{identity} candidate payload is not canonical base64") from error
        if len(payload) > maximum:
            raise ModelError(f"{identity} candidate payload exceeds its hard bound")
        if base64.b64encode(payload).decode("ascii") != value:
            raise ModelError(f"{identity} candidate payload is not canonical base64")
        payloads[identity] = payload
    return payloads


def verify_payload_binding(candidate: dict[str, Any]) -> None:
    """Rebuild every candidate derivative from its embedded source payloads."""
    source_lock = candidate["source_lock"]
    payloads = decode_candidate_payloads(candidate["payloads"], source_lock)
    for source in source_lock["sources"]:
        payload = payloads[source["id"]]
        actual = hashlib.sha256(payload).hexdigest()
        if len(payload) != source["size_bytes"] or not hmac.compare_digest(
            actual, source["sha256"]
        ):
            raise ModelError(f"{source['id']} payload is not manifest-bound")
    pinned_policy = hashlib.sha256(payloads["policy-source"]).digest()
    current_policy = hashlib.sha256(payloads["policy-current"]).digest()
    if not hmac.compare_digest(pinned_policy, current_policy):
        raise ModelError("current policy differs from the reviewed source commit")

    operations, cargo, summary = observe(source_lock, payloads)
    if operations.decode("ascii") != candidate["operations_tsv"]:
        raise ModelError("operations inventory was not derived from payloads")
    if cargo.decode("ascii") != candidate["cargo_tsv"]:
        raise ModelError("Cargo inventory was not derived from payloads")
    for section in ("openapi", "cargo", "policy"):
        if summary[section] != source_lock[section]:
            raise ModelError(f"{section} summary was not derived from payloads")

    empty = {category: [] for category in CATEGORIES}
    base = provider_lock(source_lock, empty)
    rebuilt = validate_observation(build_observation(base, payloads))
    validate_stable_cargo_matches(rebuilt)
    if candidate["observation"] != rebuilt:
        raise ModelError("observation was not derived from payloads")
    if candidate["provider_lock"] != provider_lock(
        source_lock, rebuilt["contracts"]
    ):
        raise ModelError("provider lock was not derived from payloads")


def write_once(path: Path, payload: bytes) -> None:
    if len(payload) > MAX_CANDIDATE_BYTES:
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


def unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result = {}
    for key, value in pairs:
        if key in result:
            raise ModelError("crates.io refresh candidate contains duplicate keys")
        result[key] = value
    return result


def main() -> int:
    args = parse_args()
    try:
        if args.command == "stage":
            candidate = build_candidate(args.source_commit, args.reviewed_at)
            write_once(args.output, canonical_bytes(candidate) + b"\n")
            print(f"staged crates.io refresh candidate at {args.output}")
        else:
            payload = read_bounded_bytes(
                args.candidate, "crates.io refresh candidate", MAX_CANDIDATE_BYTES
            )
            try:
                candidate = json.loads(payload, object_pairs_hook=unique_object)
            except (UnicodeDecodeError, json.JSONDecodeError) as error:
                raise ModelError(
                    "crates.io refresh candidate is not strict UTF-8 JSON"
                ) from error
            if not isinstance(candidate, dict):
                raise ModelError("crates.io refresh candidate root must be an object")
            validate_candidate(candidate)
            print("crates.io refresh candidate is complete and internally clean")
    except (ModelError, OSError, TypeError, ValueError) as error:
        print(f"crates.io refresh: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
