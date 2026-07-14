"""Deterministic grouped reports for Hetzner API fingerprint drift."""

from __future__ import annotations

import json
import sys
from typing import Any

MAX_REPORTED_ITEMS = 50


def safe_text(value: Any) -> str:
    if not isinstance(value, str):
        value = str(value)
    return json.dumps(value, ensure_ascii=True)[1:-1]


def format_key(key: tuple[str, ...]) -> str:
    return " ".join(safe_text(part) for part in key)


def row_key(row: dict[str, str], fields: tuple[str, ...]) -> tuple[str, ...]:
    return tuple(row[field] for field in fields)


def index_rows(
    name: str,
    rows: list[dict[str, str]],
    keys: tuple[str, ...],
) -> dict[tuple[str, ...], dict[str, str]]:
    indexed: dict[tuple[str, ...], dict[str, str]] = {}
    for row in rows:
        if any(
            not isinstance(field, str) or not isinstance(value, str)
            for field, value in row.items()
        ):
            raise SystemExit(f"invalid {name} row: fields and values must be text")
        try:
            key = row_key(row, keys)
        except KeyError as error:
            missing = safe_text(error.args[0])
            raise SystemExit(
                f"invalid {name} row: missing identity field {missing}"
            ) from error
        if key in indexed:
            raise SystemExit(f"duplicate {name} identity: {format_key(key)}")
        indexed[key] = row
    return indexed


def compare_row_sets(
    name: str,
    locked: list[dict[str, str]],
    current: list[dict[str, str]],
    keys: tuple[str, ...],
) -> dict[str, Any]:
    locked_map = index_rows(f"locked {name}", locked, keys)
    current_map = index_rows(f"current {name}", current, keys)
    changed: list[dict[str, Any]] = []
    for key in sorted(set(current_map) & set(locked_map)):
        if current_map[key] != locked_map[key]:
            fields = {
                field: (locked_map[key].get(field, ""), current_map[key].get(field, ""))
                for field in sorted(set(locked_map[key]) | set(current_map[key]))
                if locked_map[key].get(field) != current_map[key].get(field)
            }
            changed.append({"key": key, "fields": fields})
    return {
        "added": sorted(set(current_map) - set(locked_map)),
        "removed": sorted(set(locked_map) - set(current_map)),
        "changed": changed,
    }


def build_drift_report(
    locked_operations: list[dict[str, str]],
    current_operations: list[dict[str, str]],
    locked_schemas: list[dict[str, str]],
    current_schemas: list[dict[str, str]],
    source_hashes: dict[str, str],
    pinned_hashes: dict[str, str],
) -> dict[str, Any]:
    operations = compare_row_sets(
        "operation",
        locked_operations,
        current_operations,
        ("api", "method", "path"),
    )
    schemas = compare_row_sets(
        "schema", locked_schemas, current_schemas, ("api", "schema")
    )
    deprecated = []
    changed = []
    for change in operations["changed"]:
        fields = change["fields"]
        became_deprecated = fields.get("deprecated") == ("no", "yes")
        if became_deprecated:
            deprecated.append(change)
        other_fields = set(fields) - {"deprecated"}
        if not became_deprecated or other_fields:
            changed.append(change)

    changed_sources = []
    for api in sorted(pinned_hashes):
        actual = source_hashes.get(api, "")
        expected = pinned_hashes[api]
        if actual != expected:
            changed_sources.append(
                {"key": (api,), "fields": {"sha256": (expected, actual)}}
            )

    return {
        "added": operations["added"],
        "removed": operations["removed"],
        "deprecated": deprecated,
        "changed": changed,
        "changed_sources": changed_sources,
        "schema_only": schemas,
    }


def report_has_drift(report: dict[str, Any]) -> bool:
    schemas = report["schema_only"]
    return any(
        (
            report["added"],
            report["removed"],
            report["deprecated"],
            report["changed"],
            report["changed_sources"],
            schemas["added"],
            schemas["removed"],
            schemas["changed"],
        )
    )


def print_keys(
    label: str, keys: list[tuple[str, ...]], *, indent: int = 2
) -> None:
    if not keys:
        return
    prefix = " " * indent
    item_prefix = " " * (indent + 2)
    print(f"{prefix}{label}:", file=sys.stderr)
    for key in keys[:MAX_REPORTED_ITEMS]:
        print(f"{item_prefix}{format_key(key)}", file=sys.stderr)
    if len(keys) > MAX_REPORTED_ITEMS:
        print(
            f"{item_prefix}... {len(keys) - MAX_REPORTED_ITEMS} more omitted",
            file=sys.stderr,
        )


def print_changes(
    label: str, changes: list[dict[str, Any]], *, indent: int = 2
) -> None:
    if not changes:
        return
    prefix = " " * indent
    item_prefix = " " * (indent + 2)
    field_prefix = " " * (indent + 4)
    print(f"{prefix}{label}:", file=sys.stderr)
    for change in changes[:MAX_REPORTED_ITEMS]:
        print(f"{item_prefix}{format_key(change['key'])}", file=sys.stderr)
        for field, (before, after) in change["fields"].items():
            print(
                f"{field_prefix}{safe_text(field)}: "
                f"{safe_text(before)} -> {safe_text(after)}",
                file=sys.stderr,
            )
    if len(changes) > MAX_REPORTED_ITEMS:
        print(
            f"{item_prefix}... {len(changes) - MAX_REPORTED_ITEMS} more omitted",
            file=sys.stderr,
        )


def print_drift_report(report: dict[str, Any]) -> int:
    if not report_has_drift(report):
        print("Hetzner API: no drift")
        return 0

    print("Hetzner API: drift detected", file=sys.stderr)
    print_keys("added operations", report["added"])
    print_keys("removed operations", report["removed"])
    print_changes("deprecated operations", report["deprecated"])
    print_changes("changed operations", report["changed"])
    print_changes("changed source digests", report["changed_sources"])
    schemas = report["schema_only"]
    if schemas["added"] or schemas["removed"] or schemas["changed"]:
        print("  schema-only changes:", file=sys.stderr)
        print_keys("added schemas", schemas["added"], indent=4)
        print_keys("removed schemas", schemas["removed"], indent=4)
        print_changes("changed schemas", schemas["changed"], indent=4)
    return 1
