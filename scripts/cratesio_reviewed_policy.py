#!/usr/bin/env python3
"""Exact reviewed crates.io data-access policy boundary."""

from __future__ import annotations

import hashlib
import hmac
import re
from typing import Any

from cratesio_source_error import SourceLockError


EXPECTED_POLICY: dict[str, Any] = {
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
}


def reviewed_policy(source_payload: bytes, expected_sha256: str) -> dict[str, Any]:
    """Return the typed policy only for the exact reviewed source bytes."""
    actual = hashlib.sha256(source_payload).hexdigest()
    if not hmac.compare_digest(actual, expected_sha256):
        raise SourceLockError(
            "policy bytes changed; explicit policy review is required"
        )
    return {
        key: list(value) if isinstance(value, list) else value
        for key, value in EXPECTED_POLICY.items()
    }


def policy_observation(
    deployed_payload: bytes,
    source_payload: bytes,
    expected_sha256: str,
) -> dict[str, Any]:
    """Validate the deployed route and exact reviewed policy source."""
    try:
        deployed = deployed_payload.decode("utf-8")
    except UnicodeDecodeError as error:
        raise SourceLockError("deployed data-access route is not UTF-8") from error
    visible = " ".join(re.sub(r"<[^>]+>", " ", deployed).split())
    if "crates.io: Rust Package Registry" not in visible:
        raise SourceLockError(
            "deployed data-access route is not the crates.io application"
        )
    return reviewed_policy(source_payload, expected_sha256)
