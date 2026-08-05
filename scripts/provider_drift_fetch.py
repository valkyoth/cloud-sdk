#!/usr/bin/env python3
"""Exact-URL bounded HTTPS fetching for provider source locks."""

from __future__ import annotations

import hashlib
import ssl
import time
import urllib.request
from typing import Any, Callable
from urllib.parse import urlsplit


CONNECT_TIMEOUT_SECONDS = 10
TOTAL_TIMEOUT_SECONDS = 60
READ_CHUNK_BYTES = 64 * 1024
MAX_TOTAL_SOURCE_BYTES = 128 * 1024 * 1024


class FetchError(RuntimeError):
    """An authenticated provider source could not be obtained."""


class RejectRedirects(urllib.request.HTTPRedirectHandler):
    """Refuse every redirect before a second request is constructed."""

    def redirect_request(
        self,
        _request: Any,
        _file: Any,
        _code: int,
        _message: str,
        _headers: Any,
        _new_url: str,
    ) -> None:
        return None


def validate_response(response: Any, expected_url: str, source_id: str) -> None:
    final_url = response.geturl()
    if not isinstance(final_url, str) or urlsplit(final_url).scheme != "https":
        raise FetchError(f"{source_id} resolved to a non-HTTPS URL")
    if final_url != expected_url:
        raise FetchError(f"{source_id} redirected away from its pinned URL")


def read_bounded(
    response: Any,
    source_id: str,
    max_bytes: int,
    *,
    total_seconds: int = TOTAL_TIMEOUT_SECONDS,
    monotonic: Callable[[], float] = time.monotonic,
) -> bytes:
    started = monotonic()
    payload = bytearray()
    while True:
        if monotonic() - started > total_seconds:
            raise FetchError(f"{source_id} download exceeded {total_seconds} seconds")
        remaining = max_bytes + 1 - len(payload)
        chunk = response.read(min(READ_CHUNK_BYTES, remaining))
        if monotonic() - started > total_seconds:
            raise FetchError(f"{source_id} download exceeded {total_seconds} seconds")
        if not chunk:
            return bytes(payload)
        payload.extend(chunk)
        if len(payload) > max_bytes:
            raise FetchError(f"{source_id} exceeds {max_bytes} bytes")


def fetch_source(source: dict[str, Any]) -> bytes:
    source_id = source["id"]
    url = source["url"]
    opener = urllib.request.build_opener(
        urllib.request.HTTPSHandler(context=ssl.create_default_context()),
        RejectRedirects(),
    )
    try:
        with opener.open(url, timeout=CONNECT_TIMEOUT_SECONDS) as response:
            validate_response(response, url, source_id)
            payload = read_bounded(response, source_id, source["max_bytes"])
    except OSError as error:
        raise FetchError(f"could not fetch {source_id}: {error}") from error
    actual = hashlib.sha256(payload).hexdigest()
    if actual != source["sha256"]:
        raise FetchError(
            f"{source_id} SHA-256 mismatch: expected {source['sha256']}, got {actual}"
        )
    return payload


def with_verified_sources(
    lock: dict[str, Any], parser: Callable[[dict[str, bytes]], Any]
) -> Any:
    """Invoke a built-in adapter only after every source authenticates."""
    try:
        admitted = sum(source["max_bytes"] for source in lock["sources"])
    except (KeyError, TypeError) as error:
        raise FetchError("source fetch plan is incomplete") from error
    if admitted > MAX_TOTAL_SOURCE_BYTES:
        raise FetchError("source fetch plan exceeds its aggregate byte bound")
    payloads: dict[str, bytes] = {}
    for source in lock["sources"]:
        payloads[source["id"]] = fetch_source(source)
    return parser(payloads)
