#!/usr/bin/env python3
"""Security tests for provider source fetching."""

from __future__ import annotations

import hashlib
import io

import provider_drift_fetch as fetch


class Response(io.BytesIO):
    def __init__(self, payload: bytes, status: int = 200) -> None:
        super().__init__(payload)
        self.status = status
        self.read_calls = 0

    def read(self, size: int = -1) -> bytes:
        self.read_calls += 1
        return super().read(size)

    def __enter__(self):
        return self

    def __exit__(self, *_args) -> None:
        self.close()


class Connection:
    def __init__(self, response) -> None:
        self.response = response
        self.requests = []
        self.closed = False

    def request(self, method: str, target: str, *, headers: dict) -> None:
        self.requests.append((method, target, headers))

    def getresponse(self):
        if isinstance(self.response, Exception):
            raise self.response
        return self.response

    def close(self) -> None:
        self.closed = True


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


def test_redirect_response_is_rejected_without_reading_or_following() -> None:
    response = Response(b"redirect payload", 302)
    connection = Connection(response)
    assert_raises(
        "redirected away",
        fetch.fetch_source,
        "hetzner",
        source(b"{}"),
        resolver=global_resolver,
        connection_factory=lambda _target: connection,
    )
    assert response.read_calls == 0
    assert len(connection.requests) == 1
    assert connection.closed


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
    connections = []

    def factory(_target):
        connection = Connection(Response(payload))
        connections.append(connection)
        return connection

    assert fetch.fetch_source(
        "hetzner", source(payload), resolver=global_resolver,
        connection_factory=factory,
    ) == payload
    assert connections[0].requests == [
        (
            "GET",
            "/cloud.spec.json",
            {
                "Accept-Encoding": "identity",
                "Connection": "close",
                "Host": "docs.hetzner.cloud",
                "User-Agent": "cloud-sdk-provider-drift/0.57",
            },
        )
    ]
    wrong = source(payload)
    wrong["sha256"] = "0" * 64
    try:
        fetch.fetch_source(
            "hetzner", wrong, resolver=global_resolver,
            connection_factory=factory,
        )
    except fetch.FetchError as error:
        message = str(error)
        assert message == "cloud-openapi SHA-256 mismatch"
        assert hashlib.sha256(payload).hexdigest() not in message
    else:
        raise AssertionError("expected FetchError")


def test_transport_failure_has_bounded_error_and_closes_connection() -> None:
    connection = Connection(OSError("sensitive transport detail"))
    assert_raises(
        "could not fetch cloud-openapi",
        fetch.fetch_source,
        "hetzner",
        source(b"{}"),
        resolver=global_resolver,
        connection_factory=lambda _target: connection,
    )
    assert connection.closed


def test_pinned_connection_uses_validated_socket_and_original_sni() -> None:
    calls = []

    class RawSocket:
        def settimeout(self, timeout: int) -> None:
            calls.append(("timeout", timeout))

        def connect(self, address) -> None:
            calls.append(("connect", address))

        def close(self) -> None:
            calls.append(("close",))

    class Context:
        def wrap_socket(self, raw_socket, *, server_hostname: str):
            calls.append(("sni", server_hostname))
            return raw_socket

    target = fetch.validate_network_target(
        "hetzner", source(b"{}"), resolver=global_resolver
    )
    connection = fetch.PinnedHTTPSConnection(
        target,
        context=Context(),
        timeout=fetch.CONNECT_TIMEOUT_SECONDS,
        socket_factory=lambda family, kind, protocol: (
            calls.append(("socket", family, kind, protocol)) or RawSocket()
        ),
    )
    connection.connect()
    assert ("connect", ("93.184.216.34", 443)) in calls
    assert ("sni", "docs.hetzner.cloud") in calls
    connection.close()

    socket_attempted = False

    def unexpected_socket(*_args):
        nonlocal socket_attempted
        socket_attempted = True
        raise AssertionError("connection started after its deadline")

    expired_ticks = iter((0.0, 11.0))
    expired = fetch.PinnedHTTPSConnection(
        target,
        context=Context(),
        timeout=10,
        socket_factory=unexpected_socket,
        monotonic=lambda: next(expired_ticks),
    )
    try:
        expired.connect()
    except OSError as error:
        assert str(error) == "all validated source addresses failed"
    else:
        raise AssertionError("expired connection deadline was accepted")
    assert not socket_attempted


def test_pinned_connection_shares_one_deadline_across_addresses_and_tls() -> None:
    calls = []

    class RawSocket:
        def __init__(self, fails: bool) -> None:
            self.fails = fails

        def settimeout(self, timeout: float) -> None:
            calls.append(("timeout", timeout))

        def connect(self, address) -> None:
            calls.append(("connect", address))
            if self.fails:
                raise OSError("fixture connect failure")

        def close(self) -> None:
            calls.append(("close",))

    class Context:
        def wrap_socket(self, raw_socket, *, server_hostname: str):
            calls.append(("sni", server_hostname))
            return raw_socket

    target = fetch.ResolvedTarget(
        "docs.hetzner.cloud",
        443,
        "/cloud.spec.json",
        (
            (fetch.socket.AF_INET, fetch.socket.SOCK_STREAM, 6, ("8.8.8.8", 443)),
            (fetch.socket.AF_INET, fetch.socket.SOCK_STREAM, 6, ("8.8.4.4", 443)),
        ),
    )
    sockets = iter((RawSocket(True), RawSocket(False)))
    ticks = iter((100.0, 101.0, 106.0, 109.0))
    connection = fetch.PinnedHTTPSConnection(
        target,
        context=Context(),
        timeout=10,
        socket_factory=lambda *_args: next(sockets),
        monotonic=lambda: next(ticks),
    )
    connection.connect()
    assert [call for call in calls if call[0] == "timeout"] == [
        ("timeout", 9.0),
        ("timeout", 4.0),
        ("timeout", 1.0),
    ]
    assert ("sni", "docs.hetzner.cloud") in calls
    connection.close()


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


def test_aggregate_admission_precedes_fetch() -> None:
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


def test_network_targets_require_reviewed_global_destinations() -> None:
    item = source(b"{}")
    target = fetch.validate_network_target(
        "hetzner", item, resolver=global_resolver
    )
    assert target.host == "docs.hetzner.cloud"
    assert target.request_target == "/cloud.spec.json"
    assert target.addresses == (
        (fetch.socket.AF_INET, fetch.socket.SOCK_STREAM, 6, ("93.184.216.34", 443)),
    )
    wrong_port = fetch.validate_network_target(
        "hetzner",
        item,
        resolver=lambda *_args, **_kwargs: [
            (fetch.socket.AF_INET, fetch.socket.SOCK_STREAM, 6, "", ("93.184.216.34", 80))
        ],
    )
    assert wrong_port.addresses[0][3] == ("93.184.216.34", 443)
    duplicates = fetch.validate_network_target(
        "hetzner",
        item,
        resolver=lambda *_args, **_kwargs: global_resolver("", 0) * 32,
    )
    assert len(duplicates.addresses) == 1
    assert_raises(
        "too many addresses",
        fetch.validate_network_target,
        "hetzner",
        item,
        resolver=lambda *_args, **_kwargs: [
            (
                fetch.socket.AF_INET,
                fetch.socket.SOCK_STREAM,
                6,
                "",
                (f"8.8.8.{last}", 443),
            )
            for last in range(1, fetch.MAX_RESOLVED_ADDRESSES + 2)
        ],
    )
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
    assert_raises(
        "invalid socket type",
        fetch.validate_network_target,
        "hetzner",
        item,
        resolver=lambda *_args, **_kwargs: [
            (fetch.socket.AF_INET, fetch.socket.SOCK_DGRAM, 17, "", ("93.184.216.34", 443))
        ],
    )


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
