#!/usr/bin/env python3
"""Validate and compare provider-generic source-lock evidence."""

from __future__ import annotations

import argparse
import multiprocessing
import sys
from pathlib import Path

from provider_drift_adapters import AdapterError, build_live_observation
import provider_drift_fetch as fetch
from provider_drift_model import (
    ModelError,
    canonical_bytes,
    read_bounded_json,
    validate_lock,
    validate_observation,
    validate_plugin,
)
from provider_drift_report import build_report


PLAN_TIMEOUT_SECONDS = 180
MAX_WORKER_RESULT_BYTES = 2 * 1024 * 1024


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--plugin", required=True)
    parser.add_argument("--lock", required=True)
    parser.add_argument("--observation", required=True)
    parser.add_argument("--fetch-sources", action="store_true")
    return parser.parse_args()


def evaluate(plugin: dict, lock: dict, tracked_observation: dict) -> dict:
    expected_plugin = {"id": plugin["id"], "version": plugin["version"]}
    if (
        lock["plugin"] != expected_plugin
        or tracked_observation["plugin"] != expected_plugin
    ):
        raise ModelError("lock and observation must use the selected plugin exactly")
    return build_report(lock, tracked_observation)


def _verification_worker(
    lock: dict, tracked_observation: dict, connection: object
) -> None:
    message = b"\x00"
    try:
        payloads = fetch._fetch_verified_sources(lock)
        observation = validate_observation(build_live_observation(lock, payloads))
        if canonical_bytes(observation) != canonical_bytes(tracked_observation):
            raise ModelError("live adapter observation differs from tracked evidence")
        report = build_report(lock, observation)
        encoded = canonical_bytes(report)
        if len(encoded) > MAX_WORKER_RESULT_BYTES:
            raise ModelError("provider drift report exceeds its IPC bound")
        result = b"\x01" if report["result"] == "clean" else b"\x02"
        message = result + encoded
    except BaseException:
        pass
    try:
        connection.send_bytes(message)
    except (OSError, ValueError):
        pass
    finally:
        connection.close()


def _stop_worker(process: object) -> None:
    if process.is_alive():
        process.terminate()
        process.join(1)
    if process.is_alive():
        process.kill()
        process.join()


def verify_live_sources(
    lock: dict,
    tracked_observation: dict,
    *,
    timeout: int = PLAN_TIMEOUT_SECONDS,
    context: object | None = None,
) -> tuple[bytes, bool]:
    """Fetch, normalize, compare, and report inside one deadline worker."""
    fetch.preflight_sources(lock)
    worker_context = context or multiprocessing.get_context("spawn")
    receiver = None
    sender = None
    process = None
    try:
        receiver, sender = worker_context.Pipe(duplex=False)
        process = worker_context.Process(
            target=_verification_worker, args=(lock, tracked_observation, sender)
        )
        process.start()
    except Exception as error:
        if receiver is not None:
            receiver.close()
        if sender is not None:
            sender.close()
        if process is not None:
            _stop_worker(process)
        raise fetch.FetchError("provider source verification could not start") from error
    sender.close()
    try:
        try:
            ready = receiver.poll(timeout)
        except OSError as error:
            raise fetch.FetchError("provider source verification failed") from error
        if not ready:
            _stop_worker(process)
            raise fetch.FetchError("provider source verification exceeded its deadline")
        try:
            message = receiver.recv_bytes(MAX_WORKER_RESULT_BYTES + 1)
        except (EOFError, OSError) as error:
            raise fetch.FetchError("provider source verification failed") from error
    finally:
        receiver.close()
        process.join(1)
        _stop_worker(process)
    if len(message) < 2 or message[0] not in (1, 2):
        raise fetch.FetchError("provider source verification failed")
    return message[1:], message[0] == 1


def main() -> int:
    args = parse_args()
    try:
        plugin = validate_plugin(read_bounded_json(Path(args.plugin), "plugin"))
        lock = validate_lock(read_bounded_json(Path(args.lock), "provider lock"))
        observation = validate_observation(
            read_bounded_json(Path(args.observation), "provider observation")
        )
        if args.fetch_sources:
            expected_plugin = {"id": plugin["id"], "version": plugin["version"]}
            if lock["plugin"] != expected_plugin or observation["plugin"] != expected_plugin:
                raise ModelError(
                    "lock and observation must use the selected plugin exactly"
                )
            encoded, clean = verify_live_sources(lock, observation)
        else:
            report = evaluate(plugin, lock, observation)
            encoded = canonical_bytes(report)
            clean = report["result"] == "clean"
    except (AdapterError, fetch.FetchError, ModelError) as error:
        print(f"provider drift: {error}", file=sys.stderr)
        return 2
    sys.stdout.buffer.write(encoded + b"\n")
    return 0 if clean else 1


if __name__ == "__main__":
    raise SystemExit(main())
