#!/usr/bin/env python3
"""Report source-locked crates.io semantic drift."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from cratesio_drift_adapter import (
    CratesioAdapterError,
    build_observation,
    validate_stable_cargo_matches,
)
from cratesio_drift_documents import provider_lock, source_projection
from cratesio_source_error import SourceLockError
from cratesio_source_fetch import observe_source
from cratesio_source_manifest import validate_lock as validate_source_lock
from provider_drift_model import (
    ModelError,
    canonical_bytes,
    read_bounded_json,
    validate_lock,
    validate_observation,
    validate_plugin,
)
from provider_drift_report import build_report


ROOT = Path(__file__).resolve().parents[1]
PLUGIN = ROOT / "provider-drift/plugins/normalized-json-v1.json"
SOURCE_LOCK = ROOT / "provider-drift/providers/cratesio-source.lock.json"
LOCK = ROOT / "provider-drift/providers/cratesio.lock.json"
OBSERVATION = ROOT / "provider-drift/providers/cratesio.observed.json"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--fetch", action="store_true", help="observe current official source bytes"
    )
    return parser.parse_args()


def load_documents() -> tuple[dict, dict, dict, dict]:
    plugin = validate_plugin(read_bounded_json(PLUGIN, "plugin"))
    source_lock = read_bounded_json(SOURCE_LOCK, "crates.io source lock")
    validate_source_lock(source_lock)
    lock = validate_lock(read_bounded_json(LOCK, "crates.io provider lock"))
    observation = validate_observation(
        read_bounded_json(OBSERVATION, "crates.io provider observation")
    )
    expected_plugin = {"id": plugin["id"], "version": plugin["version"]}
    if lock["plugin"] != expected_plugin or observation["plugin"] != expected_plugin:
        raise ModelError("crates.io drift documents use the wrong plugin")
    projected = source_projection(source_lock)
    if lock["sources"] != projected or observation["sources"] != projected:
        raise ModelError("crates.io drift sources differ from the accepted source lock")
    if lock != provider_lock(source_lock, observation["contracts"]):
        raise ModelError("crates.io drift lock policy differs from the reviewed policy")
    validate_stable_cargo_matches(observation)
    return plugin, source_lock, lock, observation


def evaluate(fetch: bool) -> dict:
    _plugin, source_lock, lock, observation = load_documents()
    accepted = build_report(lock, observation)
    if accepted["result"] != "clean":
        raise ModelError("tracked crates.io drift evidence is not clean")
    if not fetch:
        return accepted
    payloads = {
        source["id"]: observe_source(source) for source in source_lock["sources"]
    }
    current = validate_observation(build_observation(lock, payloads))
    return build_report(lock, current)


def main() -> int:
    args = parse_args()
    try:
        report = evaluate(args.fetch)
    except (
        CratesioAdapterError,
        ModelError,
        OSError,
        SourceLockError,
        UnicodeError,
        json.JSONDecodeError,
    ) as error:
        print(f"crates.io drift: {error}", file=sys.stderr)
        return 2
    sys.stdout.buffer.write(canonical_bytes(report) + b"\n")
    return 0 if report["result"] == "clean" else 1


if __name__ == "__main__":
    raise SystemExit(main())
