#!/usr/bin/env python3
"""Exact-URL bounded HTTPS fetching for provider source locks."""

from __future__ import annotations

import hashlib
import ipaddress
import socket
import ssl
import time
import urllib.request
from typing import Any, Callable
from urllib.parse import urlsplit


CONNECT_TIMEOUT_SECONDS = 10
TOTAL_TIMEOUT_SECONDS = 60
READ_CHUNK_BYTES = 64 * 1024
MAX_TOTAL_SOURCE_BYTES = 128 * 1024 * 1024

APPROVED_SOURCE_ENDPOINTS = {
    ("hetzner", "cloud-openapi"): (
        "docs.hetzner.cloud",
        443,
        "https://docs.hetzner.cloud/cloud.spec.json",
    ),
    ("hetzner", "dns-openapi"): (
        "docs.hetzner.cloud",
        443,
        "https://docs.hetzner.cloud/hetzner.spec.json",
    ),
    ("ovhcloud-v2-probe", "api-index"): (
        "api.eu.ovhcloud.com",
        443,
        "https://api.eu.ovhcloud.com/v2",
    ),
    ("ovhcloud-v2-probe", "iam-schema"): (
        "api.eu.ovhcloud.com",
        443,
        "https://api.eu.ovhcloud.com/v2/iam.json",
    ),
    ("ovhcloud-v2-probe", "api-v2-principles"): (
        "raw.githubusercontent.com",
        443,
        "https://raw.githubusercontent.com/ovh/ovhcloud-docs/develop/docs/en/guides/manage-and-operate/api/apiv2.mdx",
    ),
    ("ovhcloud-v2-probe", "oauth2-service-account"): (
        "raw.githubusercontent.com",
        443,
        "https://raw.githubusercontent.com/ovh/ovhcloud-docs/develop/docs/en/guides/account-and-service-management/account-information/authenticate-api-with-service-account.mdx",
    ),
}


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


def validate_network_target(
    provider: str,
    source: dict[str, Any],
    *,
    resolver: Callable[..., Any] = socket.getaddrinfo,
) -> None:
    approved = APPROVED_SOURCE_ENDPOINTS.get((provider, source["id"]))
    if approved is None:
        raise FetchError("source endpoint has not received code review")
    host, port, approved_url = approved
    parsed = urlsplit(source["url"])
    effective_port = parsed.port if parsed.port is not None else 443
    if (
        source["url"] != approved_url
        or parsed.hostname != host
        or effective_port != port
    ):
        raise FetchError(f"{source['id']} does not use its approved endpoint")
    try:
        addresses = resolver(host, port, type=socket.SOCK_STREAM)
    except OSError as error:
        raise FetchError(f"could not resolve {source['id']}") from error
    if not addresses:
        raise FetchError(f"could not resolve {source['id']}")
    for result in addresses:
        try:
            address = ipaddress.ip_address(result[4][0])
        except (IndexError, TypeError, ValueError) as error:
            raise FetchError(f"{source['id']} resolved to an invalid address") from error
        if not address.is_global:
            raise FetchError(f"{source['id']} resolved to a non-global address")


def fetch_source(
    provider: str,
    source: dict[str, Any],
    *,
    resolver: Callable[..., Any] = socket.getaddrinfo,
) -> bytes:
    source_id = source["id"]
    url = source["url"]
    validate_network_target(provider, source, resolver=resolver)
    opener = urllib.request.build_opener(
        urllib.request.ProxyHandler({}),
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
        raise FetchError(f"{source_id} SHA-256 mismatch")
    return payload


def preflight_sources(lock: dict[str, Any]) -> None:
    try:
        admitted = sum(source["max_bytes"] for source in lock["sources"])
    except (KeyError, TypeError) as error:
        raise FetchError("source fetch plan is incomplete") from error
    if admitted > MAX_TOTAL_SOURCE_BYTES:
        raise FetchError("source fetch plan exceeds its aggregate byte bound")


def _fetch_verified_sources(lock: dict[str, Any]) -> dict[str, bytes]:
    preflight_sources(lock)
    payloads: dict[str, bytes] = {}
    for source in lock["sources"]:
        payloads[source["id"]] = fetch_source(lock["provider"], source)
    return payloads
