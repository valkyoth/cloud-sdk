#!/usr/bin/env python3
"""Credential-free bounded retrieval for Scaleway inventory evidence."""

from __future__ import annotations

import json
from pathlib import Path
import re
import ssl
import subprocess
import sys
import urllib.request
from urllib.parse import urlsplit

MAX_SOURCE = 10 * 1024 * 1024
MAX_TOTAL = 128 * 1024 * 1024
SDK_REVISION = "684f67323db64f059ef53f4c53a970c0b1591a1d"
SDK_TREE = f"https://api.github.com/repos/scaleway/scaleway-sdk-go/git/trees/{SDK_REVISION}?recursive=1"


class InventoryError(ValueError):
    """Evidence is incomplete, ambiguous, or outside its reviewed bounds."""


def approved_url(url: str) -> None:
    if not isinstance(url, str) or len(url) > 1024 or any(ord(c) <= 32 for c in url):
        raise InventoryError("invalid source URL")
    parts = urlsplit(url)
    if (parts.scheme != "https" or parts.username is not None or parts.password is not None
            or parts.port is not None or parts.fragment):
        raise InventoryError("source authority is not approved")
    if url == SDK_TREE:
        return
    if parts.query:
        raise InventoryError("source query is not approved")
    if parts.hostname == "www.scaleway.com" and re.fullmatch(
        r"/en/developers/(?:api(?:/[a-z0-9_/-]+(?:/schema\.yml)?)?|assets/[A-Za-z0-9_.-]+\.js)",
        parts.path,
    ) and "//" not in parts.path:
        return
    if parts.hostname == "raw.githubusercontent.com" and re.fullmatch(
        rf"/scaleway/scaleway-sdk-go/{SDK_REVISION}/api/[a-z0-9_/-]+\.go", parts.path
    ) and "//" not in parts.path:
        return
    raise InventoryError("source URL is outside the admitted public documentation origins")


class NoRedirects(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args: object, **kwargs: object) -> None:
        raise InventoryError("source redirect rejected; review the canonical URL")


def read_bounded(response: object, limit: int) -> bytes:
    announced = response.headers.get("Content-Length")
    if announced is not None and (not announced.isascii() or not announced.isdecimal()
                                  or int(announced) > limit):
        raise InventoryError("invalid or oversized source content length")
    if response.headers.get("Content-Encoding", "identity") != "identity":
        raise InventoryError("compressed source response rejected")
    data = bytearray()
    while True:
        chunk = response.read(min(65536, limit + 1 - len(data)))
        if not chunk:
            return bytes(data)
        data.extend(chunk)
        if len(data) > limit:
            raise InventoryError("source byte limit exceeded")


def worker(url: str, limit: int) -> bytes:
    approved_url(url)
    if type(limit) is not int or not 0 < limit <= MAX_SOURCE:
        raise InventoryError("invalid source byte budget")
    opener = urllib.request.build_opener(
        urllib.request.ProxyHandler({}),
        urllib.request.HTTPSHandler(context=ssl.create_default_context()), NoRedirects(),
    )
    request = urllib.request.Request(url, headers={
        "User-Agent": "cloud-sdk-scaleway-inventory/1.1.0",
        "Accept-Encoding": "identity",
    })
    with opener.open(request, timeout=10) as response:
        if response.status != 200 or response.geturl() != url:
            raise InventoryError("unexpected source status or final URL")
        return read_bounded(response, limit)


def fetch(url: str, limit: int = MAX_SOURCE, *, timeout: float = 60) -> bytes:
    approved_url(url)
    if type(limit) is not int or not 0 < limit <= MAX_SOURCE:
        raise InventoryError("invalid source byte budget")
    try:
        result = subprocess.run(
            [sys.executable, "-E", "-s", str(Path(__file__).resolve()), "--worker"],
            input=json.dumps([url, limit]).encode("ascii"), stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL, check=True, timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        raise InventoryError("source retrieval deadline exceeded; worker killed and reaped") from None
    except (OSError, subprocess.CalledProcessError):
        raise InventoryError(f"public source retrieval failed: {url}") from None
    if len(result.stdout) > limit:
        raise InventoryError("source worker exceeded byte budget")
    return result.stdout


if __name__ == "__main__":
    if sys.argv[1:] != ["--worker"]:
        raise SystemExit("internal source-fetch worker")
    try:
        url, limit = json.loads(sys.stdin.buffer.read(4096))
        sys.stdout.buffer.write(worker(url, limit))
    except Exception:
        raise SystemExit(1) from None
