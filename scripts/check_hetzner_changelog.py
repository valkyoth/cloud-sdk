#!/usr/bin/env python3
"""Verify the reviewed official Hetzner changelog RSS source."""

from __future__ import annotations

import argparse
import hashlib
import json
import ssl
import sys
import time
import urllib.request
from pathlib import Path
from typing import Any
from urllib.parse import urlsplit
from xml.etree import ElementTree


ROOT = Path(__file__).resolve().parents[1]
LOCK_DOCUMENT = ROOT / "docs" / "HETZNER_CHANGELOG_LOCK.md"
SOURCE_URL = "https://docs.hetzner.cloud/changelog/feed.rss"
SELF_URL = "https://docs.hetzner.cloud/changelog/rss"
PINNED_SEMANTIC_SHA256 = (
    "102fa2a3092c24a2c5cf9b25e7b7a60093ce3d6eb05ec0aa505c37843c2a6641"
)
PINNED_LATEST_GUID = (
    "https://docs.hetzner.cloud/changelog#"
    "2026-08-31-debian-11-image-is-deprecated"
)
MAX_SOURCE_BYTES = 8 * 1024 * 1024
CONNECT_TIMEOUT_SECONDS = 10
TOTAL_TIMEOUT_SECONDS = 60
READ_CHUNK_BYTES = 64 * 1024
ATOM_NAMESPACE = "{http://www.w3.org/2005/Atom}"


class RejectRedirects(urllib.request.HTTPRedirectHandler):
    """Prevent the source check from following redirects."""

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


def read_bounded_response(
    response: Any,
    *,
    max_bytes: int = MAX_SOURCE_BYTES,
    total_seconds: int = TOTAL_TIMEOUT_SECONDS,
    monotonic: Any = time.monotonic,
) -> bytes:
    """Read one bounded response under a total elapsed-time limit."""
    started = monotonic()
    data = bytearray()
    while True:
        if monotonic() - started > total_seconds:
            raise SystemExit(
                f"Hetzner changelog download exceeded {total_seconds} seconds"
            )
        remaining = max_bytes + 1 - len(data)
        chunk = response.read(min(READ_CHUNK_BYTES, remaining))
        if monotonic() - started > total_seconds:
            raise SystemExit(
                f"Hetzner changelog download exceeded {total_seconds} seconds"
            )
        if not chunk:
            return bytes(data)
        data.extend(chunk)
        if len(data) > max_bytes:
            raise SystemExit(f"Hetzner changelog exceeds {max_bytes} bytes")


def validate_response(response: Any) -> None:
    """Require the exact reviewed HTTPS source without redirects."""
    final_url = response.geturl()
    if not isinstance(final_url, str) or urlsplit(final_url).scheme.lower() != "https":
        raise SystemExit("Hetzner changelog resolved to a non-HTTPS URL")
    if final_url != SOURCE_URL:
        raise SystemExit("Hetzner changelog redirected away from its pinned URL")


def semantic_node(element: Any, path: tuple[str, ...] = ()) -> Any:
    """Return a canonical feed tree without the volatile build timestamp."""
    current_path = (*path, element.tag)
    if current_path == ("rss", "channel", "lastBuildDate"):
        return None
    children = [
        child
        for child in (semantic_node(value, current_path) for value in list(element))
        if child is not None
    ]
    return [
        element.tag,
        sorted(element.attrib.items()),
        (element.text or "").strip(),
        children,
    ]


def semantic_digest(root: Any) -> str:
    """Hash all structured feed content except its volatile build timestamp."""
    payload = json.dumps(
        semantic_node(root),
        ensure_ascii=False,
        separators=(",", ":"),
    ).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def parse_feed(payload: bytes) -> tuple[str, str, int, str]:
    """Validate the RSS identity and return the latest reviewed item metadata."""
    if b"<!DOCTYPE" in payload or b"<!ENTITY" in payload:
        raise SystemExit("Hetzner changelog XML declarations are not permitted")
    try:
        root = ElementTree.fromstring(payload)
    except ElementTree.ParseError as error:
        raise SystemExit(f"Hetzner changelog is invalid XML: {error}") from error
    if root.tag != "rss" or root.attrib != {"version": "2.0"}:
        raise SystemExit("Hetzner changelog RSS root changed")
    channels = root.findall("channel")
    if len(channels) != 1:
        raise SystemExit("Hetzner changelog channel is missing or ambiguous")
    channel = channels[0]
    if channel.findtext("title") != "Hetzner Cloud Changelog":
        raise SystemExit("Hetzner changelog title changed")
    if channel.findtext("link") != "https://docs.hetzner.cloud/changelog":
        raise SystemExit("Hetzner changelog canonical link changed")
    build_dates = channel.findall("lastBuildDate")
    if len(build_dates) != 1 or not (build_dates[0].text or "").strip():
        raise SystemExit("Hetzner changelog build timestamp is missing or ambiguous")
    self_links = channel.findall(f"{ATOM_NAMESPACE}link")
    if len(self_links) != 1 or self_links[0].attrib != {
        "href": SELF_URL,
        "rel": "self",
        "type": "application/rss+xml",
    }:
        raise SystemExit("Hetzner changelog self link changed")
    items = channel.findall("item")
    if not items:
        raise SystemExit("Hetzner changelog contains no entries")
    guids = [item.findtext("guid") for item in items]
    if any(not guid for guid in guids) or len(set(guids)) != len(guids):
        raise SystemExit("Hetzner changelog entry identities are missing or duplicated")
    title = items[0].findtext("title")
    if not title:
        raise SystemExit("Hetzner changelog latest title is missing")
    return guids[0] or "", title, len(items), semantic_digest(root)


def fetch() -> bytes:
    """Fetch the exact official source with platform TLS validation."""
    opener = urllib.request.build_opener(
        urllib.request.HTTPSHandler(context=ssl.create_default_context()),
        RejectRedirects(),
    )
    try:
        with opener.open(SOURCE_URL, timeout=CONNECT_TIMEOUT_SECONDS) as response:
            validate_response(response)
            return read_bounded_response(response)
    except OSError as error:
        raise SystemExit(f"could not fetch Hetzner changelog: {error}") from error


def validate_local_lock() -> int:
    """Check that the reviewed lock document and script agree."""
    try:
        text = LOCK_DOCUMENT.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        print(f"Hetzner changelog lock could not be read: {error}", file=sys.stderr)
        return 1
    required = (SOURCE_URL, PINNED_SEMANTIC_SHA256, PINNED_LATEST_GUID)
    missing = [value for value in required if value not in text]
    if missing:
        print("Hetzner changelog lock metadata is incomplete", file=sys.stderr)
        return 1
    print("Hetzner changelog lock metadata is current.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--local-only", action="store_true")
    mode.add_argument("--fetch", action="store_true")
    args = parser.parse_args()
    if args.local_only:
        return validate_local_lock()

    payload = fetch()
    latest_guid, latest_title, item_count, actual = parse_feed(payload)
    if actual != PINNED_SEMANTIC_SHA256 or latest_guid != PINNED_LATEST_GUID:
        print("Hetzner changelog drift detected", file=sys.stderr)
        print(f"expected semantic sha256: {PINNED_SEMANTIC_SHA256}", file=sys.stderr)
        print(f"actual semantic sha256:   {actual}", file=sys.stderr)
        print(f"latest entry: {latest_guid} ({latest_title})", file=sys.stderr)
        return 1
    print(
        "Hetzner changelog: no drift "
        f"({item_count} entries; latest: {latest_title})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
