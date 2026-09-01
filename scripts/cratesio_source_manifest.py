#!/usr/bin/env python3
"""Validate the authority-bound crates.io source manifest and artifacts."""

from __future__ import annotations

import hashlib
import hmac
import re
from typing import Any

from cratesio_source_error import SourceLockError


SOURCE_DETAILS = {
    "openapi": ("application/json", "application/json", 1048576),
    "cargo": ("text/html", "text/html", 524288),
    "policy": ("text/html", "text/html", 131072),
    "openapi-source": ("text/plain", "text/plain", 131072),
    "policy-source": ("text/plain", "text/plain", 131072),
    "policy-current": ("text/plain", "text/plain", 131072),
}


def official_urls(source_commit: str) -> dict[str, str]:
    repository = (
        "https://raw.githubusercontent.com/rust-lang/crates.io/" + source_commit
    )
    return {
        "openapi": "https://crates.io/api/openapi.json",
        "cargo": "https://doc.rust-lang.org/cargo/reference/registry-web-api.html",
        "policy": "https://crates.io/data-access",
        "openapi-source": repository + "/src/openapi.rs",
        "policy-source": (
            repository + "/svelte/src/routes/data-access/%2Bpage.svelte"
        ),
        "policy-current": (
            "https://raw.githubusercontent.com/rust-lang/crates.io/"
            "main/svelte/src/routes/data-access/%2Bpage.svelte"
        ),
    }


def validate_lock(lock: dict[str, Any]) -> None:
    required = {
        "format",
        "reviewed_at",
        "source_commit",
        "sources",
        "artifacts",
        "openapi",
        "cargo",
        "policy",
    }
    if set(lock) != required or lock.get("format") != 1:
        raise SourceLockError("crates.io source lock has unknown or missing fields")
    source_commit = lock.get("source_commit")
    if not isinstance(source_commit, str) or not re.fullmatch(
        r"[0-9a-f]{40}", source_commit
    ):
        raise SourceLockError("crates.io source commit is invalid")
    if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", str(lock.get("reviewed_at", ""))):
        raise SourceLockError("crates.io review date is invalid")
    sources = lock.get("sources")
    if not isinstance(sources, list) or len(sources) != len(SOURCE_DETAILS):
        raise SourceLockError("crates.io source lock must contain six sources")
    expected_urls = official_urls(source_commit)
    ids: set[str] = set()
    fields = {
        "id",
        "url",
        "accept",
        "final_url",
        "redirects",
        "media_type",
        "max_bytes",
        "size_bytes",
        "sha256",
    }
    for source in sources:
        if not isinstance(source, dict) or set(source) != fields:
            raise SourceLockError("crates.io source entry is incomplete")
        identity = source["id"]
        if identity in ids or identity not in expected_urls:
            raise SourceLockError("crates.io source identity is invalid")
        ids.add(identity)
        if (
            source["url"] != expected_urls[identity]
            or source["final_url"] != expected_urls[identity]
            or source["redirects"] != []
        ):
            raise SourceLockError(f"{identity} is not an approved official source")
        accept, media_type, maximum = SOURCE_DETAILS[identity]
        if (
            source["accept"] != accept
            or source["media_type"] != media_type
            or source["max_bytes"] != maximum
        ):
            raise SourceLockError(f"{identity} retrieval policy changed")
        if not re.fullmatch(r"[0-9a-f]{64}", str(source["sha256"])):
            raise SourceLockError("crates.io source digest is invalid")
        if not isinstance(source["size_bytes"], int) or not (
            0 < source["size_bytes"] <= maximum
        ):
            raise SourceLockError("crates.io source size is invalid")
    if ids != set(SOURCE_DETAILS):
        raise SourceLockError("crates.io source identities changed")
    artifacts = lock.get("artifacts")
    if not isinstance(artifacts, dict) or set(artifacts) != {
        "operations_sha256",
        "cargo_compatibility_sha256",
    }:
        raise SourceLockError("crates.io artifact digests are incomplete")
    if any(not re.fullmatch(r"[0-9a-f]{64}", str(value)) for value in artifacts.values()):
        raise SourceLockError("crates.io artifact digest is invalid")
    if lock["openapi"] != {
        "version": "3.1.0",
        "paths": 40,
        "operations": 51,
        "auth_schemes": ["api_token", "cookie", "trustpub_token"],
    }:
        raise SourceLockError("crates.io OpenAPI summary changed")
    if lock["cargo"] != {"stable_operations": 7, "instruction_targets": 1}:
        raise SourceLockError("Cargo compatibility summary changed")
    if lock["policy"] != {
        "api_max_requests_per_second": 1,
        "identifying_user_agent_required": True,
        "contact_information_recommended": True,
        "api_is_fallback": True,
        "preferred_sources": [
            "sparse-index",
            "static-downloads",
            "rss",
            "database-dumps",
        ],
    }:
        raise SourceLockError("crates.io access policy summary changed")


def validate_artifact_digests(
    lock: dict[str, Any], operations: bytes, cargo: bytes
) -> None:
    expected = lock["artifacts"]
    for label, payload, key in (
        ("OpenAPI inventory", operations, "operations_sha256"),
        ("Cargo compatibility inventory", cargo, "cargo_compatibility_sha256"),
    ):
        actual = hashlib.sha256(payload).hexdigest()
        if not hmac.compare_digest(actual, expected[key]):
            raise SourceLockError(f"{label} digest changed")
