#!/usr/bin/env python3
"""Bind OVHcloud production task routes and models to reviewed evidence."""

from __future__ import annotations

import csv
import json
import sys
from pathlib import Path

from provider_drift_model import read_bounded_json, validate_lock


ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "provider-drift/providers/ovhcloud-v2-probe.lock.json"
FIXTURE = ROOT / "crates/cloud-sdk/tests/fixtures/ovhcloud-task-contracts.tsv"
FIELDS = (
    "collection_path",
    "resource_path",
    "action",
    "collection_response",
    "resource_response",
    "cursor_headers",
    "task_fields",
    "progress_fields",
    "error_fields",
    "property_contracts",
    "statuses",
    "model_sha256",
    "schema_sha256",
    "generic_event_path",
    "generic_event_scope",
)


def fixture_row(path: Path = FIXTURE) -> dict[str, str]:
    try:
        with path.open("r", encoding="ascii", newline="") as handle:
            reader = csv.DictReader(handle, delimiter="\t")
            if tuple(reader.fieldnames or ()) != FIELDS:
                raise ValueError("task fixture fields are invalid")
            rows = list(reader)
    except (OSError, UnicodeError, csv.Error) as error:
        raise ValueError("task fixture could not be read") from error
    if len(rows) != 1:
        raise ValueError("task fixture must contain exactly one row")
    return rows[0]


def one(rows: list[dict], row_id: str) -> dict:
    matches = [row["values"] for row in rows if row.get("id") == row_id]
    if len(matches) != 1:
        raise ValueError(f"{row_id} contract is ambiguous")
    return matches[0]


def expected_row(lock: dict) -> dict[str, str]:
    contracts = lock["contracts"]
    collection = one(
        contracts["operations"], "notification/contactmean/by-contactmeanid/task"
    )
    resource = one(
        contracts["operations"],
        "notification/contactmean/by-contactmeanid/task/by-taskid",
    )
    models = one(contracts["schemas"], "notification-task-models")
    generic = one(contracts["schemas"], "task-event-contract")
    if collection["actions"] != resource["actions"] or len(collection["actions"]) != 1:
        raise ValueError("task routes do not share one reviewed read action")
    if (
        collection["method"] != "GET"
        or resource["method"] != "GET"
        or collection["stability"] != "production"
        or resource["stability"] != "production"
        or not collection["authenticated"]
        or not resource["authenticated"]
        or resource["headers"]
    ):
        raise ValueError("task routes are not stable authenticated reads")
    if any(row["values"]["path"] == generic["event_path"] for row in contracts["operations"]):
        raise ValueError("generic event example was promoted to an endpoint claim")
    return {
        "collection_path": collection["path"],
        "resource_path": resource["path"],
        "action": collection["actions"][0],
        "collection_response": collection["response_type"],
        "resource_response": resource["response_type"],
        "cursor_headers": ",".join(collection["headers"]),
        "task_fields": ",".join(models["task_fields"]),
        "progress_fields": ",".join(models["progress_fields"]),
        "error_fields": ",".join(models["error_fields"]),
        "property_contracts": json.dumps(
            models["property_contracts"],
            ensure_ascii=True,
            sort_keys=True,
            separators=(",", ":"),
        ),
        "statuses": ",".join(models["statuses"]),
        "model_sha256": models["model_sha256"],
        "schema_sha256": models["source_sha256"],
        "generic_event_path": generic["event_path"],
        "generic_event_scope": "fixture-only",
    }


def check() -> None:
    lock = validate_lock(read_bounded_json(LOCK, "OVHcloud probe lock"))
    if fixture_row() != expected_row(lock):
        raise ValueError("task fixture differs from source-locked contracts")


def main() -> int:
    try:
        check()
    except (KeyError, OSError, TypeError, UnicodeError, ValueError) as error:
        print(f"OVHcloud task conformance: {error}", file=sys.stderr)
        return 1
    print("OVHcloud production task routes and models are source-bound.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
