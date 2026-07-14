#!/usr/bin/env python3
"""Compare current Hetzner OpenAPI specs against the locked SDK fingerprints."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import os
import ssl
import stat
import sys
import tempfile
import time
import urllib.request
from urllib.parse import urlsplit
from pathlib import Path
from typing import Any

from hetzner_drift_report import (
    build_drift_report,
    compare_row_sets,
    print_drift_report,
)

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


class RejectRedirects(urllib.request.HTTPRedirectHandler):
    """Prevent the release fetch from following any redirect."""

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
    parameters = operation.get("parameters", [])
    if not isinstance(parameters, list):
        raise ValueError("parameters must be an array")
    names = set()
    for parameter in parameters:
        if not isinstance(parameter, dict):
            raise ValueError("parameter must be an object")
        if parameter.get("in") == "query":
            name = parameter.get("name")
            if not isinstance(name, str):
                raise ValueError("query parameter name must be text")
            names.add(name)
    return names


def operation_rows(api: str, document: dict[str, Any]) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    paths = document.get("paths", {})
    if not isinstance(paths, dict):
        raise SystemExit(f"{api} spec paths must be an object")
    for path, path_item in paths.items():
        if not isinstance(path, str) or not isinstance(path_item, dict):
            raise SystemExit(f"{api} spec contains an invalid path item")
        for method, operation in path_item.items():
            if method not in HTTP_METHODS:
                continue
            if not isinstance(operation, dict):
                raise SystemExit(f"{api} spec contains an invalid operation")
            try:
                queries = query_names(operation)
            except ValueError as error:
                raise SystemExit(
                    f"{api} spec contains invalid operation parameters"
                ) from error
            tags = operation.get("tags") or ["untagged"]
            operation_id = operation.get("operationId", "")
            if (
                not isinstance(tags, list)
                or not tags
                or not isinstance(tags[0], str)
                or not isinstance(operation_id, str)
            ):
                raise SystemExit(f"{api} spec contains invalid operation metadata")
            fingerprint_input = dict(operation)
            fingerprint_input.pop("deprecated", None)
            rows.append(
                {
                    "api": api,
                    "method": method.upper(),
                    "path": path,
                    "tag": tags[0],
                    "operation_id": operation_id,
                    "deprecated": "yes" if operation.get("deprecated") else "no",
                    "pagination": "yes"
                    if {"page", "per_page"}.issubset(queries)
                    else "no",
                    "sorting": "yes" if "sort" in queries else "no",
                    "action": action_kind(method, path),
                    "fingerprint": digest(fingerprint_input),
                }
            )
    return sorted(rows, key=lambda row: (row["api"], row["path"], row["method"]))


def schema_rows(api: str, document: dict[str, Any]) -> list[dict[str, str]]:
    components = document.get("components", {})
    if not isinstance(components, dict):
        raise SystemExit(f"{api} spec components must be an object")
    schemas = components.get("schemas", {})
    if not isinstance(schemas, dict) or any(
        not isinstance(name, str) for name in schemas
    ):
        raise SystemExit(f"{api} spec schemas must be a text-keyed object")
    rows = [
        {"api": api, "schema": name, "fingerprint": digest(schema)}
        for name, schema in schemas.items()
    ]
    return sorted(rows, key=lambda row: (row["api"], row["schema"]))


def read_bounded_file(
    api: str, path: Path, *, max_bytes: int = MAX_SPEC_BYTES
) -> bytes:
    required = ("O_CLOEXEC", "O_NOFOLLOW", "O_NONBLOCK")
    if any(not hasattr(os, name) for name in required):
        raise SystemExit("platform lacks secure no-follow local spec reads")
    flags = os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW | os.O_NONBLOCK
    try:
        descriptor = os.open(path, flags)
    except OSError as error:
        raise SystemExit(
            f"{api} spec must be a readable regular file: {path}"
        ) from error
    try:
        info = os.fstat(descriptor)
        if not stat.S_ISREG(info.st_mode):
            raise SystemExit(f"{api} spec must be a regular file: {path}")
        if info.st_size > max_bytes:
            raise SystemExit(f"{api} spec exceeds {max_bytes} bytes")

        data = bytearray()
        while True:
            remaining = max_bytes + 1 - len(data)
            try:
                chunk = os.read(descriptor, min(READ_CHUNK_BYTES, remaining))
            except OSError as error:
                raise SystemExit(f"{api} spec could not be read: {path}") from error
            if not chunk:
                return bytes(data)
            data.extend(chunk)
            if len(data) > max_bytes:
                raise SystemExit(f"{api} spec exceeds {max_bytes} bytes")
    finally:
        os.close(descriptor)


def parse_spec(api: str, payload: bytes) -> dict[str, Any]:
    try:
        document = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SystemExit(f"{api} spec is not valid UTF-8 JSON: {error}") from error
    if not isinstance(document, dict):
        raise SystemExit(f"{api} spec root must be a JSON object")
    return document


def read_spec(
    api: str, path: Path, *, expected_sha256: str | None
) -> tuple[dict[str, Any], str]:
    payload = read_bounded_file(api, path)
    actual = hashlib.sha256(payload).hexdigest()
    if expected_sha256 is not None and actual != expected_sha256:
        raise SystemExit(
            f"{api} spec SHA-256 mismatch: "
            f"expected {expected_sha256}, got {actual}"
        )
    return parse_spec(api, payload), actual


def read_verified_spec(api: str, path: Path) -> dict[str, Any]:
    document, _actual = read_spec(
        api, path, expected_sha256=PINNED_SPEC_SHA256[api]
    )
    return document


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


def validate_fetch_response(response: Any, expected_url: str, api: str) -> None:
    final_url = response.geturl()
    if not isinstance(final_url, str) or urlsplit(final_url).scheme.lower() != "https":
        raise SystemExit(f"{api} spec download resolved to a non-HTTPS URL")
    if final_url != expected_url:
        raise SystemExit(f"{api} spec download redirected away from its pinned URL")


def fetch_spec(api: str, directory: Path) -> Path:
    target = directory / f"{api}.spec.json"
    opener = urllib.request.build_opener(
        urllib.request.HTTPSHandler(context=ssl.create_default_context()),
        RejectRedirects(),
    )
    try:
        with opener.open(
            SPECS[api],
            timeout=FETCH_CONNECT_TIMEOUT_SECONDS,
        ) as response:
            validate_fetch_response(response, SPECS[api], api)
            payload = read_bounded_response(response, api)
    except OSError as error:
        raise SystemExit(f"could not fetch {api} spec: {error}") from error
    target.write_bytes(payload)
    print(f"{api} spec sha256: {hashlib.sha256(payload).hexdigest()}")
    return target


def load_specs(
    args: argparse.Namespace,
) -> tuple[dict[str, dict[str, Any]], dict[str, str]]:
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
    documents: dict[str, dict[str, Any]] = {}
    source_hashes: dict[str, str] = {}
    for api, path in paths.items():
        expected = None if args.fetch else PINNED_SPEC_SHA256[api]
        documents[api], source_hashes[api] = read_spec(
            api, path, expected_sha256=expected
        )
    return documents, source_hashes


def read_tsv(path: Path) -> list[dict[str, str]]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


def write_tsv(path: Path, rows: list[dict[str, str]], fields: list[str]) -> None:
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle, delimiter="\t", fieldnames=fields, lineterminator="\n"
        )
        writer.writeheader()
        writer.writerows(rows)


def ensure_refresh_sources_pinned(source_hashes: dict[str, str]) -> None:
    mismatched = [
        api
        for api, expected in PINNED_SPEC_SHA256.items()
        if source_hashes.get(api) != expected
    ]
    if mismatched:
        raise SystemExit(
            "lock refresh requires reviewed source pins for: "
            + ", ".join(sorted(mismatched))
        )


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

    documents, source_hashes = load_specs(args)
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
        ensure_refresh_sources_pinned(source_hashes)
        status = validate_local_files()
        if status == 0:
            report = build_drift_report(
                read_tsv(OP_LOCK),
                operations,
                read_tsv(SCHEMA_LOCK),
                schemas,
                source_hashes,
                PINNED_SPEC_SHA256,
            )
            status = print_drift_report(report)
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
    report = build_drift_report(
        read_tsv(OP_LOCK),
        operations,
        read_tsv(SCHEMA_LOCK),
        schemas,
        source_hashes,
        PINNED_SPEC_SHA256,
    )
    status |= print_drift_report(report)
    return status


if __name__ == "__main__":
    raise SystemExit(main())
