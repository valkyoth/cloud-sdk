#!/usr/bin/env python3
"""Regression tests for the narrow Robot wire source lock."""

from __future__ import annotations

import copy
import importlib.util
import io
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "check_robot_wire_fixture.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("check_robot_wire_fixture", SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load Robot fixture checker")
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


def fixture():
    return checker.read_fixture()


def test_committed_fixture_passes_every_local_validator() -> None:
    value = fixture()
    checker.validate_protocol(value)
    checker.validate_requests(value)
    checker.validate_responses(value)


def test_protocol_changes_fail_closed() -> None:
    value = copy.deepcopy(fixture())
    value["protocol"]["https_only"] = False
    assert_exits("protocol facts changed", checker.validate_protocol, value)


def test_credentials_cannot_enter_request_headers() -> None:
    value = copy.deepcopy(fixture())
    value["requests"][0]["headers"]["Authorization"] = "Basic forbidden"
    assert_exits("credential-bearing headers", checker.validate_requests, value)


def test_repeated_form_order_and_cardinality_are_exact() -> None:
    value = copy.deepcopy(fixture())
    value["requests"][1]["body"] = "server%5B%5D=123.123.123.123"
    assert_exits("repeated form fields", checker.validate_requests, value)


def test_lockout_quota_maintenance_and_empty_shapes_fail_closed() -> None:
    changes = (
        ("authentication-rejection", "status", 403, "authentication rejection"),
        ("quota-error", "status", 429, "quota error"),
        ("maintenance", "body", {}, "maintenance fixture"),
        ("empty-success", "body", "{}", "empty-success fixture"),
    )
    for name, field, replacement, expected in changes:
        value = copy.deepcopy(fixture())
        response = next(item for item in value["responses"] if item["name"] == name)
        response[field] = replacement
        assert_exits(expected, checker.validate_responses, value)


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
            if timeout != checker.TIMEOUT_SECONDS:
                raise AssertionError("unexpected timeout")
            return Response()

    original = checker.urllib.request.build_opener
    checker.urllib.request.build_opener = lambda *_handlers: Opener()
    try:
        assert_exits("exceeds 8 MiB", checker.fetch_source)
    finally:
        checker.urllib.request.build_opener = original


def test_redirect_handler_never_creates_a_followup_request() -> None:
    result = checker.RejectRedirects().redirect_request(
        None, None, 302, "Found", {}, "https://example.invalid"
    )
    if result is not None:
        raise AssertionError("redirect handler admitted a follow-up request")


def main() -> int:
    tests = [
        value
        for name, value in sorted(globals().items())
        if name.startswith("test_") and callable(value)
    ]
    for test in tests:
        test()
    print(f"{len(tests)} Robot wire fixture regression tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
