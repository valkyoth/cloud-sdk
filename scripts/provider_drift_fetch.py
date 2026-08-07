#!/usr/bin/env python3
"""Exact-URL bounded HTTPS fetching for provider source locks."""

from __future__ import annotations

import hashlib
import http.client
import ipaddress
import socket
import ssl
import time
from dataclasses import dataclass
from typing import Any, Callable
from urllib.parse import urlsplit

from ovhcloud_probe_adapter import OvhcloudProbeError, source_digest

CONNECT_TIMEOUT_SECONDS = 10
TOTAL_TIMEOUT_SECONDS = 60
READ_CHUNK_BYTES = 64 * 1024
MAX_TOTAL_SOURCE_BYTES = 128 * 1024 * 1024
MAX_RESOLVED_ADDRESSES = 8

APPROVED_SOURCE_ENDPOINTS = {
    ("hetzner", "cloud-openapi"): (
        "docs.hetzner.cloud",
        443,
        "https://docs.hetzner.cloud/cloud.spec.json",
    ),
    ("hetzner", "storage-openapi"): (
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
    ("ovhcloud-v2-probe", "notification-task-schema"): (
        "api.eu.ovhcloud.com",
        443,
        "https://api.eu.ovhcloud.com/v2/notification.json",
    ),
    ("ovhcloud-v2-probe", "api-v2-principles"): (
        "raw.githubusercontent.com",
        443,
        "https://raw.githubusercontent.com/ovh/ovhcloud-docs/eb5d926b9030000cfb03386c4cbe6d60491ab63a/docs/en/guides/manage-and-operate/api/apiv2.mdx",
    ),
    ("ovhcloud-v2-probe", "oauth2-service-account"): (
        "raw.githubusercontent.com",
        443,
        "https://raw.githubusercontent.com/ovh/ovhcloud-docs/eb5d926b9030000cfb03386c4cbe6d60491ab63a/docs/en/guides/account-and-service-management/account-information/authenticate-api-with-service-account.mdx",
    ),
}


class FetchError(RuntimeError):
    """An authenticated provider source could not be obtained."""


Address = tuple[int, int, int, tuple[Any, ...]]


@dataclass(frozen=True)
class ResolvedTarget:
    """One approved authority and the exact global addresses resolved once."""

    host: str
    port: int
    request_target: str
    addresses: tuple[Address, ...]


class PinnedHTTPSConnection(http.client.HTTPSConnection):
    """Connect only to prevalidated addresses while verifying the DNS name."""

    def __init__(
        self,
        target: ResolvedTarget,
        *,
        context: ssl.SSLContext,
        timeout: int,
        socket_factory: Callable[..., socket.socket] = socket.socket,
        monotonic: Callable[[], float] = time.monotonic,
    ) -> None:
        super().__init__(
            target.host,
            target.port,
            context=context,
            timeout=timeout,
        )
        self._addresses = target.addresses
        self._socket_factory = socket_factory
        self._monotonic = monotonic

    def connect(self) -> None:
        deadline = self._monotonic() + self.timeout
        last_error: OSError | None = None
        for family, kind, protocol, address in self._addresses:
            remaining = deadline - self._monotonic()
            if remaining <= 0:
                break
            raw_socket = self._socket_factory(family, kind, protocol)
            wrapped_socket = None
            try:
                raw_socket.settimeout(remaining)
                raw_socket.connect(address)
                remaining = deadline - self._monotonic()
                if remaining <= 0:
                    raw_socket.close()
                    break
                raw_socket.settimeout(remaining)
                wrapped_socket = self._context.wrap_socket(
                    raw_socket,
                    server_hostname=self.host,
                )
                if self._monotonic() >= deadline:
                    wrapped_socket.close()
                    break
                wrapped_socket.settimeout(self.timeout)
                self.sock = wrapped_socket
                return
            except OSError as error:
                last_error = error
                if wrapped_socket is None:
                    raw_socket.close()
                else:
                    wrapped_socket.close()
        raise OSError("all validated source addresses failed") from last_error


def connection_for(target: ResolvedTarget) -> PinnedHTTPSConnection:
    """Construct the production pinned-address HTTPS connection."""
    return PinnedHTTPSConnection(
        target,
        context=ssl.create_default_context(),
        timeout=CONNECT_TIMEOUT_SECONDS,
    )


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
) -> ResolvedTarget:
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
    validated: list[Address] = []
    seen: set[Address] = set()
    for result in addresses:
        try:
            family, kind, protocol, _canonical, socket_address = result
            address = ipaddress.ip_address(socket_address[0])
        except (IndexError, TypeError, ValueError) as error:
            raise FetchError(f"{source['id']} resolved to an invalid address") from error
        if not address.is_global:
            raise FetchError(f"{source['id']} resolved to a non-global address")
        if kind != socket.SOCK_STREAM:
            raise FetchError(f"{source['id']} resolved to an invalid socket type")
        if family == socket.AF_INET and len(socket_address) == 2:
            pinned_address = (address.compressed, port)
        elif family == socket.AF_INET6 and len(socket_address) == 4:
            pinned_address = (
                address.compressed,
                port,
                socket_address[2],
                socket_address[3],
            )
        else:
            raise FetchError(f"{source['id']} resolved to an invalid address family")
        candidate = (family, kind, protocol, pinned_address)
        if candidate in seen:
            continue
        if len(validated) >= MAX_RESOLVED_ADDRESSES:
            raise FetchError(f"{source['id']} returned too many addresses")
        seen.add(candidate)
        validated.append(candidate)
    request_target = parsed.path or "/"
    if parsed.query:
        request_target = f"{request_target}?{parsed.query}"
    return ResolvedTarget(host, port, request_target, tuple(validated))


def fetch_source(
    provider: str,
    source: dict[str, Any],
    *,
    resolver: Callable[..., Any] = socket.getaddrinfo,
    connection_factory: Callable[[ResolvedTarget], Any] = connection_for,
) -> bytes:
    source_id = source["id"]
    target = validate_network_target(provider, source, resolver=resolver)
    connection = connection_factory(target)
    try:
        connection.request(
            "GET",
            target.request_target,
            headers={
                "Accept-Encoding": "identity",
                "Connection": "close",
                "Host": target.host,
                "User-Agent": "cloud-sdk-provider-drift/0.57",
            },
        )
        with connection.getresponse() as response:
            if 300 <= response.status <= 399:
                raise FetchError(f"{source_id} redirected away from its pinned URL")
            if response.status != 200:
                raise FetchError(f"{source_id} returned an unexpected HTTP status")
            payload = read_bounded(response, source_id, source["max_bytes"])
    except (OSError, http.client.HTTPException) as error:
        raise FetchError(f"could not fetch {source_id}") from error
    finally:
        connection.close()
    try:
        actual = (
            source_digest(source_id, payload)
            if provider == "ovhcloud-v2-probe"
            else hashlib.sha256(payload).hexdigest()
        )
    except OvhcloudProbeError as error:
        raise FetchError(f"{source_id} integrity normalization failed") from error
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
