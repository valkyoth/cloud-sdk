#!/usr/bin/env python3
"""Canonical payload-free provider drift reports."""

from __future__ import annotations

from typing import Any

from provider_drift_model import CATEGORIES, canonical_sha256


def _index(rows: list[dict[str, Any]]) -> dict[str, dict[str, Any]]:
    return {row["id"]: row for row in rows}


def _field_paths(old: Any, new: Any, prefix: str = "") -> list[str]:
    if type(old) is not type(new):
        return [prefix]
    if isinstance(old, dict):
        paths: list[str] = []
        for key in sorted(set(old) | set(new)):
            segment = key.replace("~", "~0").replace("/", "~1")
            path = f"{prefix}/{segment}"
            if key not in old or key not in new:
                paths.append(path)
            else:
                paths.extend(_field_paths(old[key], new[key], path))
        return paths
    if isinstance(old, list):
        if old == new:
            return []
        return [prefix]
    return [] if old == new else [prefix]


def _contract_changes(
    lock: dict[str, Any], observation: dict[str, Any]
) -> list[dict[str, Any]]:
    changes: list[dict[str, Any]] = []
    owners = lock["owners"]
    policies = lock["compatibility"]
    for category in CATEGORIES:
        expected = _index(lock["contracts"][category])
        current = _index(observation["contracts"][category])
        policy = policies[category]
        owner = owners[policy["owner"]]
        for row_id in sorted(set(expected) | set(current)):
            if row_id not in expected:
                kind = "added"
                fields = [""]
                old_digest = None
                new_digest = canonical_sha256(current[row_id]["values"])
            elif row_id not in current:
                kind = "removed"
                fields = [""]
                old_digest = canonical_sha256(expected[row_id]["values"])
                new_digest = None
            else:
                old_values = expected[row_id]["values"]
                new_values = current[row_id]["values"]
                fields = _field_paths(old_values, new_values)
                if not fields:
                    continue
                kind = "changed"
                old_digest = canonical_sha256(old_values)
                new_digest = canonical_sha256(new_values)
            changes.append(
                {
                    "category": category,
                    "fields": fields,
                    "id": row_id,
                    "kind": kind,
                    "new_sha256": new_digest,
                    "old_sha256": old_digest,
                    "owner": owner,
                    "severity": policy[kind],
                }
            )
    return changes


def _source_changes(
    lock: dict[str, Any], observation: dict[str, Any]
) -> list[dict[str, Any]]:
    changes: list[dict[str, Any]] = []
    expected = _index(lock["sources"])
    current = _index(observation["sources"])
    owner = lock["owners"]["security"]
    for source_id in sorted(set(expected) | set(current)):
        if source_id not in expected:
            kind = "added"
            fields = [""]
            old_digest = None
            new_digest = canonical_sha256(current[source_id])
        elif source_id not in current:
            kind = "removed"
            fields = [""]
            old_digest = canonical_sha256(expected[source_id])
            new_digest = None
        else:
            fields = _field_paths(expected[source_id], current[source_id])
            if not fields:
                continue
            kind = "changed"
            old_digest = canonical_sha256(expected[source_id])
            new_digest = canonical_sha256(current[source_id])
        changes.append(
            {
                "category": "sources",
                "fields": fields,
                "id": source_id,
                "kind": kind,
                "new_sha256": new_digest,
                "old_sha256": old_digest,
                "owner": owner,
                "severity": "blocking",
            }
        )
    return changes


def build_report(
    lock: dict[str, Any], observation: dict[str, Any]
) -> dict[str, Any]:
    identity_changes: list[dict[str, Any]] = []
    for field in ("provider", "plugin"):
        if lock[field] != observation[field]:
            identity_changes.append(
                {
                    "category": "identity",
                    "fields": [f"/{field}"],
                    "id": field,
                    "kind": "changed",
                    "new_sha256": canonical_sha256(observation[field]),
                    "old_sha256": canonical_sha256(lock[field]),
                    "owner": lock["owners"]["security"],
                    "severity": "blocking",
                }
            )
    changes = identity_changes + _source_changes(lock, observation)
    changes.extend(_contract_changes(lock, observation))
    changes.sort(key=lambda item: (item["category"], item["id"], item["kind"]))
    return {
        "changes": changes,
        "format": "cloud-sdk-provider-drift-report/v1",
        "provider": lock["provider"],
        "result": "clean" if not changes else "drift",
    }
