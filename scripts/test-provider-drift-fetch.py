#!/usr/bin/env python3
"""Security tests for provider source fetching."""

from __future__ import annotations

import hashlib
import io
import urllib.error

import provider_drift_fetch as fetch


class Response(io.BytesIO):
    def __init__(self, payload: bytes, url: str) -> None:
        super().__init__(payload)
        self.url = url

    def geturl(self) -> str:
        return self.url

    def __enter__(self):
        return self

    def __exit__(self, *_args) -> None:
        self.close()


class Opener:
    def __init__(self, response) -> None:
        self.response = response

    def open(self, _url: str, *, timeout: int):
        assert timeout == fetch.CONNECT_TIMEOUT_SECONDS
        if isinstance(self.response, Exception):
            raise self.response
        return self.response


def assert_raises(expected: str, function, *args, **kwargs) -> None:
    try:
        function(*args, **kwargs)
    except fetch.FetchError as error:
        assert expected in str(error), error
        return
    raise AssertionError("expected FetchError")


def source(payload: bytes) -> dict:
    return {
        "id": "fixture",
        "max_bytes": len(payload),
        "sha256": hashlib.sha256(payload).hexdigest(),
        "url": "https://example.invalid/spec.json",
    }


def test_redirect_handler_and_final_url_checks_fail_closed() -> None:
    handler = fetch.RejectRedirects()
    request = fetch.urllib.request.Request("https://example.invalid/spec.json")
    assert handler.redirect_request(
        request, None, 302, "Found", {}, "https://attacker.invalid/spec.json"
    ) is None
    assert_raises(
        "redirected away",
        fetch.validate_response,
        Response(b"{}", "https://attacker.invalid/spec.json"),
        "https://example.invalid/spec.json",
        "fixture",
    )
    assert_raises(
        "non-HTTPS",
        fetch.validate_response,
        Response(b"{}", "http://example.invalid/spec.json"),
        "https://example.invalid/spec.json",
        "fixture",
    )


def test_bounded_reader_accepts_exact_and_rejects_plus_one_and_timeout() -> None:
    assert fetch.read_bounded(io.BytesIO(b"1234"), "fixture", 4) == b"1234"
    assert_raises(
        "exceeds 4 bytes", fetch.read_bounded, io.BytesIO(b"12345"), "fixture", 4
    )
    ticks = iter((0, 0, 61))
    assert_raises(
        "exceeded 60 seconds",
        fetch.read_bounded,
        io.BytesIO(b"{}"),
        "fixture",
        8,
        monotonic=lambda: next(ticks),
    )


def test_digest_is_verified_before_payload_is_returned() -> None:
    payload = b'{"openapi":"3.0.0"}'
    original = fetch.urllib.request.build_opener
    fetch.urllib.request.build_opener = lambda *_handlers: Opener(
        Response(payload, "https://example.invalid/spec.json")
    )
    try:
        assert fetch.fetch_source(source(payload)) == payload
        wrong = source(payload)
        wrong["sha256"] = "0" * 64
        assert_raises("SHA-256 mismatch", fetch.fetch_source, wrong)
    finally:
        fetch.urllib.request.build_opener = original


def test_rejected_redirect_has_bounded_error_and_no_payload() -> None:
    url = "https://example.invalid/spec.json"
    original = fetch.urllib.request.build_opener
    fetch.urllib.request.build_opener = lambda *_handlers: Opener(
        urllib.error.HTTPError(url, 302, "Found", {}, None)
    )
    try:
        assert_raises("could not fetch fixture", fetch.fetch_source, source(b"{}"))
    finally:
        fetch.urllib.request.build_opener = original


def test_adapter_is_not_invoked_until_every_source_authenticates() -> None:
    calls: list[str] = []
    parsed: list[bool] = []
    original = fetch.fetch_source

    def staged(item: dict) -> bytes:
        calls.append(item["id"])
        if item["id"] == "second":
            raise fetch.FetchError("second SHA-256 mismatch")
        return b"first"

    fetch.fetch_source = staged
    lock = {
        "sources": [
            {"id": "first", "max_bytes": 8},
            {"id": "second", "max_bytes": 8},
        ]
    }
    try:
        assert_raises(
            "SHA-256 mismatch",
            fetch.with_verified_sources,
            lock,
            lambda _payloads: parsed.append(True),
        )
    finally:
        fetch.fetch_source = original
    assert calls == ["first", "second"]
    assert not parsed


def test_aggregate_admission_precedes_fetch_and_adapter_invocation() -> None:
    called: list[bool] = []
    lock = {
        "sources": [
            {"id": "first", "max_bytes": fetch.MAX_TOTAL_SOURCE_BYTES},
            {"id": "second", "max_bytes": 1},
        ]
    }
    assert_raises(
        "aggregate byte bound",
        fetch.with_verified_sources,
        lock,
        lambda _payloads: called.append(True),
    )
    assert not called


def main() -> None:
    tests = tuple(
        value
        for name, value in globals().items()
        if name.startswith("test_") and callable(value)
    )
    for test in tests:
        test()
    print(f"{len(tests)} provider drift fetch tests passed.")


if __name__ == "__main__":
    main()
