#!/usr/bin/env python3
"""Construct the reviewed crates.io provider-drift documents."""

from __future__ import annotations

from typing import Any

from provider_drift_model import CATEGORIES


def source_projection(source_lock: dict[str, Any]) -> list[dict[str, Any]]:
    """Project the rich source manifest into the neutral drift model."""
    return sorted(
        [
            {
                "id": source["id"],
                "max_bytes": source["max_bytes"],
                "sha256": source["sha256"],
                "url": source["url"],
            }
            for source in source_lock["sources"]
        ],
        key=lambda source: source["id"],
    )


def compatibility() -> dict[str, dict[str, str]]:
    """Return the closed review policy for each neutral category."""
    provider_categories = {"cost", "operations", "pagination", "schemas"}
    policies = {}
    for category in CATEGORIES:
        policies[category] = {
            "added": "review" if category in provider_categories else "blocking",
            "changed": "blocking",
            "owner": "provider" if category in provider_categories else "security",
            "removed": "blocking",
        }
    return policies


def provider_lock(
    source_lock: dict[str, Any], contracts: dict[str, Any]
) -> dict[str, Any]:
    """Build a validated-shape accepted lock from observed contracts."""
    return {
        "compatibility": compatibility(),
        "contracts": contracts,
        "format": "cloud-sdk-provider-lock/v1",
        "owners": {
            "provider": "cratesio-maintainers",
            "release": "release-maintainers",
            "security": "security-maintainers",
        },
        "plugin": {"id": "normalized-json", "version": 1},
        "provider": "cratesio",
        "sources": source_projection(source_lock),
    }
