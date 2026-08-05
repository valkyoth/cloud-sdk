#!/usr/bin/env python3
"""Validate and compare provider-generic source-lock evidence."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from provider_drift_adapters import AdapterError, build_live_observation
from provider_drift_fetch import FetchError, fetch_verified_sources
from provider_drift_model import (
    ModelError,
    canonical_bytes,
    read_bounded_json,
    validate_lock,
    validate_observation,
    validate_plugin,
)
from provider_drift_report import build_report


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--plugin", required=True)
    parser.add_argument("--lock", required=True)
    parser.add_argument("--observation", required=True)
    parser.add_argument("--fetch-sources", action="store_true")
    return parser.parse_args()


def evaluate(
    plugin: dict, lock: dict, tracked_observation: dict, payloads: dict | None = None
) -> dict:
    expected_plugin = {"id": plugin["id"], "version": plugin["version"]}
    if (
        lock["plugin"] != expected_plugin
        or tracked_observation["plugin"] != expected_plugin
    ):
        raise ModelError("lock and observation must use the selected plugin exactly")
    observation = tracked_observation
    if payloads is not None:
        observation = validate_observation(build_live_observation(lock, payloads))
        if canonical_bytes(observation) != canonical_bytes(tracked_observation):
            raise ModelError("live adapter observation differs from tracked evidence")
    return build_report(lock, observation)


def main() -> int:
    args = parse_args()
    try:
        plugin = validate_plugin(read_bounded_json(Path(args.plugin), "plugin"))
        lock = validate_lock(read_bounded_json(Path(args.lock), "provider lock"))
        observation = validate_observation(
            read_bounded_json(Path(args.observation), "provider observation")
        )
        payloads = fetch_verified_sources(lock) if args.fetch_sources else None
        report = evaluate(plugin, lock, observation, payloads)
    except (AdapterError, FetchError, ModelError) as error:
        print(f"provider drift: {error}", file=sys.stderr)
        return 2
    sys.stdout.buffer.write(canonical_bytes(report) + b"\n")
    return 0 if report["result"] == "clean" else 1


if __name__ == "__main__":
    raise SystemExit(main())
