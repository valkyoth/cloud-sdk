#!/usr/bin/env python3
"""Compare current Hetzner OpenAPI specs against the locked SDK fingerprints."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import sys
import tempfile
import time
import urllib.request
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
OP_LOCK = ROOT / "docs" / "API_FINGERPRINTS.tsv"
SCHEMA_LOCK = ROOT / "docs" / "API_SCHEMA_FINGERPRINTS.tsv"
MATRIX = ROOT / "docs" / "API_MATRIX.md"
SPEC_LOCK = ROOT / "docs" / "SPEC_LOCK.md"

SPECS = {
    "cloud": "https://docs.hetzner.cloud/cloud.spec.json",
    "hetzner": "https://docs.hetzner.cloud/hetzner.spec.json",
}

PINNED_SPEC_SHA256 = {
    "cloud": "9ca6b542a057b002804b9f4f45ccfdb8b9a28c92b7e5bf5ae1b7f46b54fe0093",
    "hetzner": "f70750016d81c927ddf877e103541c90d3e3372723cdf54e6fd7b2eba4a8108a",
}

DOC_ONLY_KEYS = {"description", "summary", "externalDocs", "example", "examples"}
HTTP_METHODS = {"get", "post", "put", "patch", "delete"}
MAX_SPEC_BYTES = 32 * 1024 * 1024
FETCH_CONNECT_TIMEOUT_SECONDS = 10
FETCH_TOTAL_TIMEOUT_SECONDS = 60
READ_CHUNK_BYTES = 64 * 1024


def clean_json(value: Any) -> Any:
    if isinstance(value, dict):
        return {
            key: clean_json(item)
            for key, item in sorted(value.items())
            if key not in DOC_ONLY_KEYS
        }
    if isinstance(value, list):
        return [clean_json(item) for item in value]
    return value


def digest(value: Any) -> str:
    payload = json.dumps(clean_json(value), sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()


def action_kind(method: str, path: str) -> str:
    if path.endswith("/actions"):
        return "action-list"
    if "/actions/{action_id}" in path:
        return "resource-action-get"
    if "/actions/{id}" in path:
        return "action-get"
    if "/actions/" in path and method == "post":
        return "starts-action"
    if "/actions/" in path:
        return "action"
    return "none"


def query_names(operation: dict[str, Any]) -> set[str]:
    return {
        param.get("name", "")
        for param in operation.get("parameters", [])
        if param.get("in") == "query"
    }


def operation_rows(api: str, document: dict[str, Any]) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for path, path_item in document.get("paths", {}).items():
        for method, operation in path_item.items():
            if method not in HTTP_METHODS:
                continue
            queries = query_names(operation)
            rows.append(
                {
                    "api": api,
                    "method": method.upper(),
                    "path": path,
                    "tag": (operation.get("tags") or ["untagged"])[0],
                    "operation_id": operation.get("operationId", ""),
                    "deprecated": "yes" if operation.get("deprecated") else "no",
                    "pagination": "yes"
                    if {"page", "per_page"}.issubset(queries)
                    else "no",
                    "sorting": "yes" if "sort" in queries else "no",
                    "action": action_kind(method, path),
                    "fingerprint": digest(operation),
                }
            )
    return sorted(rows, key=lambda row: (row["api"], row["path"], row["method"]))


def schema_rows(api: str, document: dict[str, Any]) -> list[dict[str, str]]:
    schemas = document.get("components", {}).get("schemas", {})
    rows = [
        {"api": api, "schema": name, "fingerprint": digest(schema)}
        for name, schema in schemas.items()
    ]
    return sorted(rows, key=lambda row: (row["api"], row["schema"]))


def read_spec(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_pinned_spec(api: str, path: Path) -> None:
    actual = file_sha256(path)
    expected = PINNED_SPEC_SHA256[api]
    if actual != expected:
        raise SystemExit(
            f"{api} spec SHA-256 mismatch: "
            f"expected {expected}, got {actual}"
        )


def read_bounded_response(
    response: Any,
    api: str,
    *,
    max_bytes: int = MAX_SPEC_BYTES,
    total_seconds: int = FETCH_TOTAL_TIMEOUT_SECONDS,
    monotonic: Any = time.monotonic,
) -> bytes:
    started = monotonic()
    data = bytearray()
    while True:
        if monotonic() - started > total_seconds:
            raise SystemExit(f"{api} spec download exceeded {total_seconds} seconds")
        remaining = max_bytes + 1 - len(data)
        chunk = response.read(min(READ_CHUNK_BYTES, remaining))
        if monotonic() - started > total_seconds:
            raise SystemExit(f"{api} spec download exceeded {total_seconds} seconds")
        if not chunk:
            break
        data.extend(chunk)
        if len(data) > max_bytes:
            raise SystemExit(f"{api} spec exceeds {max_bytes} bytes")
    return bytes(data)


def fetch_spec(api: str, directory: Path) -> Path:
    target = directory / f"{api}.spec.json"
    with urllib.request.urlopen(
        SPECS[api], timeout=FETCH_CONNECT_TIMEOUT_SECONDS
    ) as response:
        target.write_bytes(read_bounded_response(response, api))
    print(f"{api} spec sha256: {file_sha256(target)}")
    return target


def load_specs(args: argparse.Namespace) -> dict[str, dict[str, Any]]:
    paths: dict[str, Path] = {}
    if args.fetch:
        tmp = tempfile.TemporaryDirectory()
        args._tmp = tmp
        tmp_path = Path(tmp.name)
        paths = {api: fetch_spec(api, tmp_path) for api in SPECS}
    else:
        if not args.current_cloud or not args.current_hetzner:
            raise SystemExit(
                "provide --fetch or both --current-cloud and --current-hetzner"
            )
        paths = {
            "cloud": Path(args.current_cloud),
            "hetzner": Path(args.current_hetzner),
        }
    for api, path in paths.items():
        verify_pinned_spec(api, path)
    return {api: read_spec(path) for api, path in paths.items()}


def read_tsv(path: Path) -> list[dict[str, str]]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


def write_tsv(path: Path, rows: list[dict[str, str]], fields: list[str]) -> None:
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, delimiter="\t", fieldnames=fields)
        writer.writeheader()
        writer.writerows(rows)


def row_key(row: dict[str, str], fields: tuple[str, ...]) -> tuple[str, ...]:
    return tuple(row[field] for field in fields)


def compare_rows(
    name: str,
    locked: list[dict[str, str]],
    current: list[dict[str, str]],
    keys: tuple[str, ...],
) -> int:
    locked_map = {row_key(row, keys): row for row in locked}
    current_map = {row_key(row, keys): row for row in current}
    added = sorted(set(current_map) - set(locked_map))
    removed = sorted(set(locked_map) - set(current_map))
    changed = []
    for key in sorted(set(current_map) & set(locked_map)):
        if current_map[key] != locked_map[key]:
            changed.append(key)

    if not added and not removed and not changed:
        print(f"{name}: no drift")
        return 0

    print(f"{name}: drift detected", file=sys.stderr)
    for key in added[:50]:
        print(f"  added: {' '.join(key)}", file=sys.stderr)
    for key in removed[:50]:
        print(f"  removed: {' '.join(key)}", file=sys.stderr)
    for key in changed[:50]:
        print(f"  changed: {' '.join(key)}", file=sys.stderr)
        locked_row = locked_map[key]
        current_row = current_map[key]
        for field in sorted(current_row):
            if current_row[field] != locked_row.get(field):
                print(
                    f"    {field}: {locked_row.get(field, '')} -> {current_row[field]}",
                    file=sys.stderr,
                )
    if len(added) + len(removed) + len(changed) > 50:
        print("  output truncated to first 50 entries per category", file=sys.stderr)
    return 1


def validate_local_files() -> int:
    status = 0
    for path in (OP_LOCK, SCHEMA_LOCK, MATRIX, SPEC_LOCK):
        if not path.is_file() or path.stat().st_size == 0:
            print(f"missing required lock file: {path}", file=sys.stderr)
            status = 1
    if status:
        return status

    operation_count = len(read_tsv(OP_LOCK))
    schema_count = len(read_tsv(SCHEMA_LOCK))
    matrix_text = MATRIX.read_text(encoding="utf-8")
    spec_text = SPEC_LOCK.read_text(encoding="utf-8")
    required = [
        f"Total source-locked operations: {operation_count}",
        "https://docs.hetzner.cloud/cloud.spec.json",
        "https://docs.hetzner.cloud/hetzner.spec.json",
        PINNED_SPEC_SHA256["cloud"],
        PINNED_SPEC_SHA256["hetzner"],
    ]
    for text in required:
        if text not in matrix_text and text not in spec_text:
            print(f"missing required lock text: {text}", file=sys.stderr)
            status = 1
    print(f"locked operations: {operation_count}")
    print(f"locked schemas: {schema_count}")
    return status


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--local-only", action="store_true")
    parser.add_argument("--fetch", action="store_true")
    parser.add_argument("--current-cloud")
    parser.add_argument("--current-hetzner")
    parser.add_argument("--write-lock", action="store_true")
    parser.add_argument("--accept-lock-refresh", action="store_true")
    args = parser.parse_args()

    if args.local_only:
        return validate_local_files()

    documents = load_specs(args)
    operations = []
    schemas = []
    for api, document in documents.items():
        operations.extend(operation_rows(api, document))
        schemas.extend(schema_rows(api, document))

    if args.write_lock:
        if not args.accept_lock_refresh:
            raise SystemExit(
                "--write-lock requires --accept-lock-refresh after drift review"
            )
        status = validate_local_files()
        if status == 0:
            status |= compare_rows(
                "operations",
                read_tsv(OP_LOCK),
                operations,
                ("api", "method", "path"),
            )
            status |= compare_rows(
                "schemas", read_tsv(SCHEMA_LOCK), schemas, ("api", "schema")
            )
            if status:
                print("accepted drift; writing refreshed lock files")
        write_tsv(
            OP_LOCK,
            operations,
            [
                "api",
                "method",
                "path",
                "tag",
                "operation_id",
                "deprecated",
                "pagination",
                "sorting",
                "action",
                "fingerprint",
            ],
        )
        write_tsv(SCHEMA_LOCK, schemas, ["api", "schema", "fingerprint"])
        print(f"wrote {len(operations)} operation fingerprints")
        print(f"wrote {len(schemas)} schema fingerprints")
        return 0

    status = validate_local_files()
    status |= compare_rows(
        "operations", read_tsv(OP_LOCK), operations, ("api", "method", "path")
    )
    status |= compare_rows("schemas", read_tsv(SCHEMA_LOCK), schemas, ("api", "schema"))
    return status


if __name__ == "__main__":
    raise SystemExit(main())
