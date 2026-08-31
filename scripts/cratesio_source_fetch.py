#!/usr/bin/env python3
"""Bounded retrieval for crates.io source-lock evidence."""

from __future__ import annotations

import hashlib
import ssl
import time
import urllib.request
from typing import Any
from urllib.parse import urlsplit

from cratesio_source_lock import SourceLockError


class SameOriginRedirects(urllib.request.HTTPRedirectHandler):
    def __init__(self, original_url: str, maximum: int = 3) -> None:
        self.original_url = original_url
        self.maximum = maximum
        self.redirects: list[str] = []

    @staticmethod
    def _origin(url: str) -> tuple[str, str, int | None]:
        parts = urlsplit(url)
        if parts.scheme.lower() != "https" or not parts.hostname or parts.username:
            raise SourceLockError("source redirect is not an uncredentialed HTTPS URL")
        return parts.scheme.lower(), parts.hostname.lower(), parts.port

    def redirect_request(
        self,
        request: Any,
        file: Any,
        code: int,
        message: str,
        headers: Any,
        new_url: str,
    ) -> Any:
        if self._origin(new_url) != self._origin(self.original_url):
            raise SourceLockError("source redirect crosses an authority boundary")
        if len(self.redirects) >= self.maximum:
            raise SourceLockError("source redirect limit exceeded")
        self.redirects.append(new_url)
        return super().redirect_request(request, file, code, message, headers, new_url)


def fetch_source(source: dict[str, Any], monotonic: Any = time.monotonic) -> bytes:
    redirector = SameOriginRedirects(source["url"])
    opener = urllib.request.build_opener(
        urllib.request.HTTPSHandler(context=ssl.create_default_context()), redirector
    )
    request = urllib.request.Request(
        source["url"],
        headers={
            "Accept": source["accept"],
            "User-Agent": "cloud-sdk-source-lock/1.1.0",
        },
    )
    started = monotonic()
    try:
        with opener.open(request, timeout=10) as response:
            payload = read_response(
                response, source, redirector.redirects, started, monotonic
            )
    except OSError as error:
        raise SourceLockError(f"could not fetch {source['id']}: {error}") from error
    actual = hashlib.sha256(payload).hexdigest()
    if len(payload) != source["size_bytes"] or actual != source["sha256"]:
        raise SourceLockError(f"{source['id']} digest or size changed")
    return payload


def read_response(
    response: Any,
    source: dict[str, Any],
    redirects: list[str],
    started: float,
    monotonic: Any,
) -> bytes:
    if response.geturl() != source["final_url"] or redirects != source["redirects"]:
        raise SourceLockError(f"{source['id']} redirect evidence changed")
    if response.headers.get_content_type() != source["media_type"]:
        raise SourceLockError(f"{source['id']} media type changed")
    announced = response.headers.get("Content-Length")
    if announced is not None:
        try:
            length = int(announced)
        except ValueError as error:
            raise SourceLockError(f"{source['id']} has invalid content length") from error
        if length < 0 or length > source["max_bytes"]:
            raise SourceLockError(f"{source['id']} exceeds its size bound")
    data = bytearray()
    while True:
        if monotonic() - started > 60:
            raise SourceLockError(f"{source['id']} exceeded its time bound")
        chunk = response.read(min(65536, source["max_bytes"] + 1 - len(data)))
        if not chunk:
            return bytes(data)
        data.extend(chunk)
        if len(data) > source["max_bytes"]:
            raise SourceLockError(f"{source['id']} exceeds its size bound")
