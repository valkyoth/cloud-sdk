#!/usr/bin/env python3
"""Regression tests for the complete Robot API source lock."""

from __future__ import annotations

import copy
import importlib.util
from pathlib import Path
import signal
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "check_robot_api_lock.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("check_robot_api_lock", SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load Robot API checker")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


checker = load_checker()


def assert_exits(expected: str, function, *args) -> None:
    try:
        function(*args)
    except SystemExit as error:
        if expected not in str(error):
            raise AssertionError(f"expected {expected!r} in {error!r}") from error
        return
    raise AssertionError("expected SystemExit")


def lock():
    return checker.read_lock()


def operation(value, operation_id: str):
    return next(item for item in value["operations"] if item["id"] == operation_id)


def test_committed_lock_passes_every_local_validator() -> None:
    value = lock()
    checker.validate_metadata(value)
    checker.validate_operations(value)


def test_unknown_top_level_fields_fail_closed() -> None:
    value = copy.deepcopy(lock())
    value["authorization"] = "Basic example"
    assert_exits("top-level fields changed", checker.validate_metadata, value)


def test_count_and_duplicate_drift_fail_closed() -> None:
    value = copy.deepcopy(lock())
    value["operations"].pop()
    assert_exits("105 operation", checker.validate_operations, value)

    value = copy.deepcopy(lock())
    value["operations"][1]["id"] = value["operations"][0]["id"]
    assert_exits("duplicate operation id", checker.validate_operations, value)


def test_operation_id_assignments_are_cryptographically_bound() -> None:
    value = copy.deepcopy(lock())
    first = value["operations"][0]
    second = value["operations"][1]
    first["id"], second["id"] = second["id"], first["id"]
    assert_exits("reviewed operation policy changed", checker.validate_operations, value)


def test_cross_family_assignments_are_cryptographically_bound() -> None:
    value = copy.deepcopy(lock())
    server = operation(value, "list_servers")
    cancellation = operation(value, "get_server_cancellation")
    server["group"], cancellation["group"] = (
        cancellation["group"],
        server["group"],
    )
    server["milestone"], cancellation["milestone"] = (
        cancellation["milestone"],
        server["milestone"],
    )
    assert_exits("reviewed operation policy changed", checker.validate_operations, value)


def test_storage_deprecation_cannot_be_reclassified_or_reimplemented() -> None:
    value = copy.deepcopy(lock())
    storage = operation(value, "list_legacy_storage_boxes")
    storage["status"] = "active"
    assert_exits("wrong status", checker.validate_operations, value)

    value = copy.deepcopy(lock())
    storage = operation(value, "list_legacy_storage_boxes")
    storage["milestone"] = "v0.90.0"
    assert_exits("wrong milestone", checker.validate_operations, value)


def test_active_operation_milestones_are_exact() -> None:
    value = copy.deepcopy(lock())
    operation(value, "delete_subnet_cancellation")["milestone"] = "v0.81.0"
    assert_exits("wrong milestone", checker.validate_operations, value)


def test_protocol_lockout_and_retry_policy_are_exact() -> None:
    value = copy.deepcopy(lock())
    rejection = value["protocol"]["authentication_rejection"]
    rejection["automatic_retry"] = True
    assert_exits("protocol policy changed", checker.validate_metadata, value)


def test_heading_parser_preserves_official_order() -> None:
    payload = b"""
    <h2 id='one'>GET /server</h2>
    <h3>GET /ignored</h3>
    <h2 id='two'>POST /server/{server-number}</h2>
    """
    assert checker.extract_headings(payload) == [
        "GET /server",
        "POST /server/{server-number}",
    ]


def test_source_validation_rejects_digest_and_heading_drift() -> None:
    value = lock()
    assert_exits("source digest changed", checker.validate_source, value, b"changed")

    headings = [
        f"<h2>{item['method']} {item['path']}</h2>" for item in value["operations"]
    ]
    headings[0] = "<h2>GET /changed</h2>"
    payload = "\n".join(headings).encode("utf-8")
    original = checker.SOURCE_SHA256
    checker.SOURCE_SHA256 = checker.hashlib.sha256(payload).hexdigest()
    try:
        assert_exits("operation headings changed", checker.validate_source, value, payload)
    finally:
        checker.SOURCE_SHA256 = original


def test_source_validation_requires_every_deprecation_marker() -> None:
    value = lock()
    deprecated = [
        item for item in value["operations"] if item["status"] == "deprecated"
    ]
    headings = [
        f"<h2>{item['method']} {item['path']}</h2>" for item in value["operations"]
    ]
    markers = [
        f"@deprecated {item['method']} {item['path']}"
        for item in deprecated[1:]
    ]
    payload = "\n".join(headings + markers).encode("utf-8")
    original = checker.SOURCE_SHA256
    checker.SOURCE_SHA256 = checker.hashlib.sha256(payload).hexdigest()
    try:
        missing = f"@deprecated {deprecated[0]['method']} {deprecated[0]['path']}"
        assert_exits(missing, checker.validate_source, value, payload)
    finally:
        checker.SOURCE_SHA256 = original


def test_source_reader_enforces_the_byte_limit() -> None:
    class Response:
        def __enter__(self):
            return self

        def __exit__(self, *_args):
            return False

        def geturl(self):
            return checker.SOURCE_URL

        def read(self, _limit):
            return b"x" * (checker.MAX_SOURCE_BYTES + 1)

    class Opener:
        def open(self, _request, *, timeout):
            assert timeout == checker.TIMEOUT_SECONDS
            return Response()

    original = checker.urllib.request.build_opener
    checker.urllib.request.build_opener = lambda *_handlers: Opener()
    try:
        assert_exits("exceeds 8 MiB", checker.fetch_source)
    finally:
        checker.urllib.request.build_opener = original


def test_lock_reader_bounds_input_before_json_parsing() -> None:
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "oversized.json"
        path.write_bytes(b"{" + b" " * checker.MAX_LOCK_BYTES)
        original = checker.LOCK
        checker.LOCK = path
        try:
            assert_exits("lock exceeds 256 KiB", checker.read_lock)
        finally:
            checker.LOCK = original


def test_fetch_deadline_uses_a_real_wall_clock_alarm() -> None:
    if not all(
        hasattr(signal, name) for name in ("SIGALRM", "ITIMER_REAL", "setitimer")
    ):
        assert_exits("unavailable", checker.fetch_deadline().__enter__)
        return
    try:
        with checker.fetch_deadline():
            signal.raise_signal(signal.SIGALRM)
    except TimeoutError as error:
        assert "exceeded total deadline" in str(error)
    else:
        raise AssertionError("expected hard fetch deadline")


def test_redirect_handler_never_creates_a_followup_request() -> None:
    result = checker.RejectRedirects().redirect_request(
        None, None, 302, "Found", {}, "https://example.invalid"
    )
    assert result is None


def main() -> int:
    tests = [
        value
        for name, value in sorted(globals().items())
        if name.startswith("test_") and callable(value)
    ]
    for test in tests:
        test()
    print(f"{len(tests)} Robot API lock regression tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
