#!/usr/bin/env python3
"""Validate and compare provider-generic source-lock evidence."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from provider_drift_fetch import FetchError, with_verified_sources
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


def main() -> int:
    args = parse_args()
    try:
        plugin = validate_plugin(read_bounded_json(Path(args.plugin), "plugin"))
        lock = validate_lock(read_bounded_json(Path(args.lock), "provider lock"))
        observation = validate_observation(
            read_bounded_json(Path(args.observation), "provider observation")
        )
        expected_plugin = {"id": plugin["id"], "version": plugin["version"]}
        if lock["plugin"] != expected_plugin or observation["plugin"] != expected_plugin:
            raise ModelError("lock and observation must use the selected plugin exactly")
        if args.fetch_sources:
            with_verified_sources(lock, lambda _payloads: None)
        report = build_report(lock, observation)
    except (FetchError, ModelError) as error:
        print(f"provider drift: {error}", file=sys.stderr)
        return 2
    sys.stdout.buffer.write(canonical_bytes(report) + b"\n")
    return 0 if report["result"] == "clean" else 1


if __name__ == "__main__":
    raise SystemExit(main())
