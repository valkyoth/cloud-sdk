#!/usr/bin/env python3
"""Data-flow tests for live provider drift observations."""

from __future__ import annotations

import copy
from pathlib import Path

import check_provider_drift as checker
import provider_drift_adapters as adapters
from provider_drift_model import read_bounded_json, validate_lock, validate_observation
from provider_drift_model import validate_plugin


ROOT = Path(__file__).resolve().parents[1]
PLUGIN = ROOT / "provider-drift" / "plugins" / "normalized-json-v1.json"
LOCK = ROOT / "provider-drift" / "providers" / "hetzner.lock.json"
OBSERVATION = ROOT / "provider-drift" / "providers" / "hetzner.observed.json"


def fixtures() -> tuple[dict, dict, dict]:
    return (
        validate_plugin(read_bounded_json(PLUGIN, "plugin")),
        validate_lock(read_bounded_json(LOCK, "provider lock")),
        validate_observation(read_bounded_json(OBSERVATION, "provider observation")),
    )


class CaptureConnection:
    def __init__(self) -> None:
        self.message = b""
        self.closed = False

    def send_bytes(self, message: bytes) -> None:
        self.message = message

    def close(self) -> None:
        self.closed = True


def run_worker(tracked: dict, live: dict | BaseException) -> bytes:
    _plugin, lock, _observation = fixtures()
    original_fetch = checker.fetch._fetch_verified_sources
    original_adapter = checker.build_live_observation
    checker.fetch._fetch_verified_sources = lambda _lock: {
        "authenticated": b"raw-provider-secret"
    }

    def normalize(_lock: dict, _payloads: dict) -> dict:
        if isinstance(live, BaseException):
            raise live
        return copy.deepcopy(live)

    checker.build_live_observation = normalize
    connection = CaptureConnection()
    try:
        checker._verification_worker(lock, tracked, connection)
    finally:
        checker.fetch._fetch_verified_sources = original_fetch
        checker.build_live_observation = original_adapter
    assert connection.closed
    return connection.message


def test_worker_contains_fetch_normalization_comparison_and_reporting() -> None:
    _plugin, _lock, tracked = fixtures()
    message = run_worker(tracked, tracked)
    assert message[0] == 1
    assert b'"result":"clean"' in message
    assert b"raw-provider-secret" not in message

    stale = copy.deepcopy(tracked)
    stale["contracts"]["schemas"] = []
    assert run_worker(tracked, stale) == b"\x00"
    assert run_worker(tracked, UnicodeEncodeError("ascii", "x", 0, 1, "bad")) == b"\x00"
    assert run_worker(tracked, RecursionError("deep source")) == b"\x00"


def test_source_derived_categories_do_not_inherit_lock_values() -> None:
    _plugin, lock, _tracked = fixtures()
    lock["contracts"]["authentication"][0]["values"]["scheme"] = "attacker"
    lock["contracts"]["cost"] = []
    lock["contracts"]["headers"] = []
    lock["contracts"]["pagination"] = []
    lock["contracts"]["retry"] = []
    document = {
        "components": {
            "securitySchemes": {"APIToken": {"scheme": "bearer", "type": "http"}},
            "schemas": {},
        },
        "paths": {
            "/servers": {
                "get": {
                    "responses": {
                        "200": {
                            "headers": {
                                "X-Next": {"schema": {"type": "string"}}
                            }
                        }
                    }
                }
            }
        },
        "security": [{"APIToken": []}],
        "servers": [{"url": "https://api.hetzner.cloud/v1"}],
    }
    storage_document = copy.deepcopy(document)
    storage_document["servers"] = [{"url": "https://api.hetzner.com/v1"}]
    original_parse = adapters.hetzner.parse_spec
    original_operations = adapters.hetzner.operation_rows
    original_schemas = adapters.hetzner.schema_rows
    original_responses = adapters.responses.rows
    original_services = adapters.responses.operation_services
    adapters.hetzner.parse_spec = lambda api, _payload: (
        document if api == "cloud" else storage_document
    )
    adapters.hetzner.operation_rows = lambda _api, _document: []
    adapters.hetzner.schema_rows = lambda _api, _document: []
    adapters.responses.rows = lambda _api, _document: []
    adapters.responses.operation_services = lambda: {}
    try:
        observed = adapters._hetzner_observation(
            lock, {"cloud-openapi": b"cloud", "storage-openapi": b"storage"}
        )
    finally:
        adapters.hetzner.parse_spec = original_parse
        adapters.hetzner.operation_rows = original_operations
        adapters.hetzner.schema_rows = original_schemas
        adapters.responses.rows = original_responses
        adapters.responses.operation_services = original_services
    authentication = {
        row["id"]: row["values"] for row in observed["contracts"]["authentication"]
    }
    endpoints = {row["id"]: row["values"] for row in observed["contracts"]["endpoints"]}
    assert authentication["cloud-bearer"]["scheme"] == "bearer"
    assert endpoints["storage-v1"]["host"] == "api.hetzner.com"
    assert observed["contracts"]["cost"][0]["values"]["path"] == (
        "docs/OPERATION_ASSOCIATIONS.tsv"
    )
    assert len(observed["contracts"]["headers"]) == 4
    assert observed["contracts"]["headers"][0]["values"]["names"] == ["x-next"]
    assert observed["contracts"]["pagination"][0]["values"]["operation_count"] == 0
    assert observed["contracts"]["retry"][0]["values"]["delivery_phases"] == [
        "not_sent",
        "possibly_sent",
        "response_started",
    ]


class FakeReceiver:
    def __init__(self) -> None:
        self.closed = False

    def poll(self, timeout: int) -> bool:
        assert timeout == 3
        return False

    def close(self) -> None:
        self.closed = True


class FakeSender:
    def close(self) -> None:
        pass


class FakeProcess:
    def __init__(self) -> None:
        self.started = False
        self.alive = True
        self.terminated = False

    def start(self) -> None:
        self.started = True

    def is_alive(self) -> bool:
        return self.alive

    def terminate(self) -> None:
        self.terminated = True
        self.alive = False

    def kill(self) -> None:
        self.alive = False

    def join(self, _timeout=None) -> None:
        pass


class FakeContext:
    def __init__(self) -> None:
        self.receiver = FakeReceiver()
        self.process = FakeProcess()

    def Pipe(self, *, duplex: bool):
        assert not duplex
        return self.receiver, FakeSender()

    def Process(self, *, target, args):
        assert target is checker._verification_worker
        assert len(args) == 3
        return self.process


def test_whole_verification_deadline_terminates_the_worker() -> None:
    _plugin, lock, tracked = fixtures()
    context = FakeContext()
    try:
        checker.verify_live_sources(lock, tracked, timeout=3, context=context)
    except checker.fetch.FetchError as error:
        assert "exceeded its deadline" in str(error)
    else:
        raise AssertionError("expected FetchError")
    assert context.process.started
    assert context.process.terminated
    assert context.receiver.closed


def test_manifest_identity_cannot_select_an_unreviewed_adapter() -> None:
    _plugin, lock, _tracked = fixtures()
    lock["provider"] = "unreviewed"
    try:
        adapters.build_live_observation(lock, {})
    except adapters.AdapterError as error:
        assert "no reviewed source adapter" in str(error)
    else:
        raise AssertionError("expected AdapterError")


def main() -> None:
    tests = tuple(
        value
        for name, value in globals().items()
        if name.startswith("test_") and callable(value)
    )
    for test in tests:
        test()
    print(f"{len(tests)} provider drift data-flow tests passed.")


if __name__ == "__main__":
    main()
