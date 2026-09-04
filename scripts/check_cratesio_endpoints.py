#!/usr/bin/env python3
"""Validate source-locked crates.io endpoint authorities."""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import date
from pathlib import Path
from typing import Any, Callable

from cratesio_source_error import SourceLockError
from cratesio_source_fetch import fetch_source


ROOT = Path(__file__).resolve().parents[1]
LOCK = Path("provider-drift/providers/cratesio-endpoints.lock.json")
AUTHORITY = Path("crates/cloud-sdk-cratesio/src/endpoint/authority.rs")
SOURCE_COMMIT = "9ae7f769cea32f38ebc2ea9ec2ce455b47641511"
INDEX_URL = "https://index.crates.io/config.json"
STAGING_SOURCE_URL = (
    "https://raw.githubusercontent.com/rust-lang/crates.io/"
    + SOURCE_COMMIT
    + "/docs/CONTRIBUTING.md"
)
EXPECTED_SOURCES = {
    "index-config": {
        "url": INDEX_URL,
        "accept": "application/json",
        "final_url": INDEX_URL,
        "redirects": [],
        "media_type": "application/octet-stream",
        "max_bytes": 4096,
        "size_bytes": 76,
        "sha256": "5b943a2c6f7eb743f7308aba07bdbb47d9ae44aafecd832d7f15df186afbafb3",
    },
    "staging-source": {
        "url": STAGING_SOURCE_URL,
        "accept": "text/plain",
        "final_url": STAGING_SOURCE_URL,
        "redirects": [],
        "media_type": "text/plain",
        "max_bytes": 262144,
        "size_bytes": 24413,
        "sha256": "95656a69d07234654d1aeecdb2a369052c07a84295befd5f1749b139c3379651",
    },
}
EXPECTED_CONSTANTS = {
    "CRATES_IO_API_BASE_URL": "https://crates.io",
    "CRATES_IO_STAGING_API_BASE_URL": "https://staging.crates.io",
    "CRATES_IO_STATIC_DOWNLOAD_BASE_URL": "https://static.crates.io",
}


def unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise SourceLockError(f"endpoint source lock repeats {key!r}")
        result[key] = value
    return result


def load_lock(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_bytes(), object_pairs_hook=unique_object)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SourceLockError("endpoint source lock is not strict UTF-8 JSON") from error
    if not isinstance(value, dict):
        raise SourceLockError("endpoint source lock root is not an object")
    return value


def validate_manifest(lock: dict[str, Any]) -> list[dict[str, Any]]:
    if set(lock) != {"format", "reviewed_at", "source_commit", "sources"}:
        raise SourceLockError("endpoint source lock fields changed")
    if lock["format"] != "cloud-sdk-cratesio-endpoints/v1":
        raise SourceLockError("endpoint source lock format changed")
    if lock["source_commit"] != SOURCE_COMMIT:
        raise SourceLockError("endpoint source commit changed")
    reviewed = lock.get("reviewed_at")
    if not isinstance(reviewed, str):
        raise SourceLockError("endpoint source review date is invalid")
    try:
        if date.fromisoformat(reviewed).isoformat() != reviewed:
            raise SourceLockError("endpoint source review date is not canonical")
    except ValueError as error:
        raise SourceLockError("endpoint source review date is invalid") from error
    sources = lock.get("sources")
    if not isinstance(sources, list) or len(sources) != len(EXPECTED_SOURCES):
        raise SourceLockError("endpoint source inventory changed")
    found: set[str] = set()
    for source in sources:
        if not isinstance(source, dict):
            raise SourceLockError("endpoint source entry is invalid")
        identity = source.get("id")
        if not isinstance(identity, str) or identity in found:
            raise SourceLockError("endpoint source identity is invalid")
        found.add(identity)
        expected = EXPECTED_SOURCES.get(identity)
        if expected is None or source != {"id": identity, **expected}:
            raise SourceLockError(f"endpoint source {identity!r} changed")
    if found != set(EXPECTED_SOURCES):
        raise SourceLockError("endpoint source identities changed")
    return sources


def validate_index(payload: bytes) -> None:
    try:
        document = json.loads(payload, object_pairs_hook=unique_object)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SourceLockError("registry index config is not strict UTF-8 JSON") from error
    expected = {
        "api": EXPECTED_CONSTANTS["CRATES_IO_API_BASE_URL"],
        "dl": EXPECTED_CONSTANTS["CRATES_IO_STATIC_DOWNLOAD_BASE_URL"] + "/crates",
    }
    if document != expected:
        raise SourceLockError("registry index endpoint configuration changed")


def validate_staging(payload: bytes) -> None:
    try:
        text = payload.decode("utf-8")
    except UnicodeDecodeError as error:
        raise SourceLockError("staging source is not UTF-8") from error
    if text.count("https://staging.crates.io") != 1 or "pnpm dev:staging" not in text:
        raise SourceLockError("staging authority evidence changed")


def validate_constants(root: Path) -> None:
    text = (root / AUTHORITY).read_text(encoding="ascii")
    for name, value in EXPECTED_CONSTANTS.items():
        declaration = f'pub const {name}: &str = "{value}";'
        if text.count(declaration) != 1:
            raise SourceLockError(f"provider endpoint constant {name} changed")
    if re.search(r'https?://[^"\\s]+', text) is None:
        raise SourceLockError("provider endpoint constants are missing")


def validate(
    root: Path,
    *,
    live: bool = False,
    fetcher: Callable[[dict[str, Any]], bytes] = fetch_source,
) -> None:
    sources = validate_manifest(load_lock(root / LOCK))
    validate_constants(root)
    if live:
        payloads = {source["id"]: fetcher(source) for source in sources}
        validate_index(payloads["index-config"])
        validate_staging(payloads["staging-source"])


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fetch", action="store_true")
    args = parser.parse_args()
    try:
        validate(ROOT, live=args.fetch)
    except (OSError, SourceLockError) as error:
        print(f"crates.io endpoints: {error}", file=sys.stderr)
        return 1
    mode = "live sources" if args.fetch else "committed evidence"
    print(f"crates.io endpoint authorities and {mode} passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
