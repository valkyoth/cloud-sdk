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
        "id": "cloud-openapi",
        "max_bytes": len(payload),
        "sha256": hashlib.sha256(payload).hexdigest(),
        "url": "https://docs.hetzner.cloud/cloud.spec.json",
    }


def global_resolver(_host: str, _port: int, **_kwargs):
    return [(fetch.socket.AF_INET, fetch.socket.SOCK_STREAM, 6, "", ("93.184.216.34", 443))]


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
    handlers = []

    def opener(*items):
        handlers.extend(items)
        return Opener(Response(payload, "https://docs.hetzner.cloud/cloud.spec.json"))

    fetch.urllib.request.build_opener = opener
    try:
        assert (
            fetch.fetch_source("hetzner", source(payload), resolver=global_resolver)
            == payload
        )
        proxy = next(item for item in handlers if isinstance(item, fetch.urllib.request.ProxyHandler))
        assert proxy.proxies == {}
        wrong = source(payload)
        wrong["sha256"] = "0" * 64
        try:
            fetch.fetch_source("hetzner", wrong, resolver=global_resolver)
        except fetch.FetchError as error:
            message = str(error)
            assert message == "cloud-openapi SHA-256 mismatch"
            assert hashlib.sha256(payload).hexdigest() not in message
        else:
            raise AssertionError("expected FetchError")
    finally:
        fetch.urllib.request.build_opener = original


def test_rejected_redirect_has_bounded_error_and_no_payload() -> None:
    url = "https://docs.hetzner.cloud/cloud.spec.json"
    original = fetch.urllib.request.build_opener
    fetch.urllib.request.build_opener = lambda *_handlers: Opener(
        urllib.error.HTTPError(url, 302, "Found", {}, None)
    )
    try:
        assert_raises(
            "could not fetch cloud-openapi",
            fetch.fetch_source,
            "hetzner",
            source(b"{}"),
            resolver=global_resolver,
        )
    finally:
        fetch.urllib.request.build_opener = original


def test_all_sources_authenticate_before_payloads_are_returned() -> None:
    calls: list[str] = []
    original = fetch.fetch_source

    def staged(_provider: str, item: dict) -> bytes:
        calls.append(item["id"])
        if item["id"] == "second":
            raise fetch.FetchError("second SHA-256 mismatch")
        return b"first"

    fetch.fetch_source = staged
    lock = {
        "provider": "hetzner",
        "sources": [
            {"id": "first", "max_bytes": 8},
            {"id": "second", "max_bytes": 8},
        ]
    }
    try:
        assert_raises(
            "SHA-256 mismatch",
            fetch._fetch_verified_sources,
            lock,
        )
    finally:
        fetch.fetch_source = original
    assert calls == ["first", "second"]


def test_aggregate_admission_precedes_fetch_and_adapter_invocation() -> None:
    called: list[bool] = []
    lock = {
        "provider": "hetzner",
        "sources": [
            {"id": "first", "max_bytes": fetch.MAX_TOTAL_SOURCE_BYTES},
            {"id": "second", "max_bytes": 1},
        ]
    }
    assert_raises(
        "aggregate byte bound",
        fetch._fetch_verified_sources,
        lock,
    )
    assert not called


def test_network_targets_require_reviewed_global_destinations() -> None:
    item = source(b"{}")
    assert_raises(
        "non-global address",
        fetch.validate_network_target,
        "hetzner",
        item,
        resolver=lambda *_args, **_kwargs: [
            (fetch.socket.AF_INET, fetch.socket.SOCK_STREAM, 6, "", ("127.0.0.1", 443))
        ],
    )
    changed = dict(item)
    changed["url"] = "https://docs.hetzner.cloud:444/cloud.spec.json"
    assert_raises(
        "approved endpoint",
        fetch.validate_network_target,
        "hetzner",
        changed,
        resolver=global_resolver,
    )
    assert_raises(
        "code review",
        fetch.validate_network_target,
        "other-provider",
        item,
        resolver=global_resolver,
    )


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
        assert target is fetch._fetch_worker
        assert len(args) == 2
        return self.process


def test_whole_plan_deadline_terminates_the_worker() -> None:
    context = FakeContext()
    lock = {"provider": "hetzner", "sources": []}
    assert_raises(
        "hard deadline",
        fetch.fetch_verified_sources,
        lock,
        timeout=3,
        context=context,
    )
    assert context.process.started
    assert context.process.terminated
    assert context.receiver.closed


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
