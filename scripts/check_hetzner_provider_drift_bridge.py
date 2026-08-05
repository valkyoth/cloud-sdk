#!/usr/bin/env python3
"""Bind the neutral Hetzner drift bridge to its complete legacy locks."""

from __future__ import annotations

import argparse
import hashlib
import re
from pathlib import Path

import check_hetzner_api_drift as hetzner
from provider_drift_model import read_bounded_bytes, read_bounded_json, validate_lock
from provider_drift_model import validate_observation


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_LOCK = ROOT / "provider-drift" / "providers" / "hetzner.lock.json"
DEFAULT_OBSERVATION = (
    ROOT / "provider-drift" / "providers" / "hetzner.observed.json"
)
MAX_EVIDENCE_BYTES = 16 * 1024 * 1024
EXPECTED_PLUGIN = {"id": "normalized-json", "version": 1}


class BridgeError(RuntimeError):
    """The neutral bridge no longer represents the authoritative locks."""


def _rows(contracts: dict, category: str) -> dict:
    return {row["id"]: row["values"] for row in contracts[category]}


def _verify_evidence(value: dict, root: Path) -> None:
    path_value = value.get("path")
    digest = value.get("sha256")
    count_value = value.get("count")
    if (
        not isinstance(path_value, str)
        or not path_value
        or not path_value.isascii()
        or "\\" in path_value
        or re.fullmatch(r"[A-Za-z0-9._/-]+", path_value) is None
        or not isinstance(digest, str)
        or re.fullmatch(r"[0-9a-f]{64}", digest) is None
        or type(count_value) is not int
        or count_value < 0
    ):
        raise BridgeError("evidence descriptor is invalid")
    path = root / path_value
    try:
        relative = path.resolve().relative_to(root.resolve())
    except ValueError as error:
        raise BridgeError("evidence path escapes the repository") from error
    if str(relative) != path_value:
        raise BridgeError("evidence path is not canonical")
    try:
        payload = read_bounded_bytes(path, "Hetzner bridge evidence", MAX_EVIDENCE_BYTES)
    except ValueError as error:
        raise BridgeError("evidence must be a bounded regular file") from error
    actual = hashlib.sha256(payload).hexdigest()
    if actual != digest:
        raise BridgeError("evidence digest is stale")
    count = max(0, len(payload.splitlines()) - 1)
    if count != count_value:
        raise BridgeError("evidence row count is stale")


def _verify_digest_evidence(value: dict, root: Path) -> None:
    if set(value) != {"path", "sha256"}:
        raise BridgeError("policy evidence descriptor is invalid")
    descriptor = {**value, "count": 0}
    try:
        path = root / value["path"]
        payload = read_bounded_bytes(path, "Hetzner policy evidence", MAX_EVIDENCE_BYTES)
    except (TypeError, ValueError) as error:
        raise BridgeError("policy evidence must be a bounded regular file") from error
    descriptor["count"] = max(0, len(payload.splitlines()) - 1)
    _verify_evidence(descriptor, root)


def validate_bridge(lock: dict, observation: dict, root: Path = ROOT) -> None:
    if lock["provider"] != "hetzner" or observation["provider"] != "hetzner":
        raise BridgeError("bridge provider must remain Hetzner")
    if lock["plugin"] != EXPECTED_PLUGIN or observation["plugin"] != EXPECTED_PLUGIN:
        raise BridgeError("bridge plugin identity is unsupported")
    if lock["contracts"] != observation["contracts"]:
        raise BridgeError("tracked Hetzner observation differs from its lock")
    if lock["sources"] != observation["sources"]:
        raise BridgeError("tracked Hetzner source observation differs from its lock")
    sources = {source["id"]: source for source in lock["sources"]}
    expected_sources = {
        "cloud-openapi": (hetzner.SPECS["cloud"], hetzner.PINNED_SPEC_SHA256["cloud"]),
        "dns-openapi": (
            hetzner.SPECS["hetzner"],
            hetzner.PINNED_SPEC_SHA256["hetzner"],
        ),
    }
    if set(sources) != set(expected_sources):
        raise BridgeError("Hetzner source set differs from the authoritative checker")
    for source_id, (url, digest) in expected_sources.items():
        if sources[source_id]["url"] != url or sources[source_id]["sha256"] != digest:
            raise BridgeError(f"Hetzner source pin is stale: {source_id}")

    operations = _rows(lock["contracts"], "operations")
    schemas = _rows(lock["contracts"], "schemas")
    policies = (
        _rows(lock["contracts"], "cost")["operation-policy"],
        _rows(lock["contracts"], "idempotency")["operation-policy"],
        _rows(lock["contracts"], "retry")["operation-policy"],
    )
    _verify_evidence(operations["active-operation-lock"], root)
    _verify_evidence(operations["response-binding-lock"], root)
    _verify_evidence(schemas["openapi-schema-lock"], root)
    for policy in policies:
        evidence = {
            "count": 208,
            "path": policy["path"],
            "sha256": policy["sha256"],
        }
        _verify_evidence(evidence, root)
    headers = _rows(lock["contracts"], "headers")
    for evidence in headers["response-metadata-policy"]["evidence"]:
        _verify_digest_evidence(evidence, root)
    _verify_digest_evidence(headers["rate-limit-policy"]["evidence"], root)
    if operations["active-operation-lock"]["active_count"] != 208:
        raise BridgeError("active Hetzner operation count is stale")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--lock", default=str(DEFAULT_LOCK))
    parser.add_argument("--observation", default=str(DEFAULT_OBSERVATION))
    args = parser.parse_args()
    try:
        lock = validate_lock(read_bounded_json(Path(args.lock), "provider lock"))
        observation = validate_observation(
            read_bounded_json(Path(args.observation), "provider observation")
        )
        validate_bridge(lock, observation)
    except (KeyError, TypeError):
        raise SystemExit("Hetzner provider drift bridge: evidence shape is invalid") from None
    except (BridgeError, ValueError) as error:
        raise SystemExit(f"Hetzner provider drift bridge: {error}") from error
    print("Hetzner provider drift bridge matches every authoritative lock.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
