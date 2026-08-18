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
from generate_response_operations import render as render_response_operations
from generate_response_operations import rows as response_operation_rows
from generate_cloud_model_schema import render as render_cloud_model_schema
from generate_cloud_model_schema import render_fixtures as render_cloud_model_fixtures
from generate_request_contract_inventory import (
    INVENTORY_OUTPUT as REQUEST_INVENTORY_LOCK,
    OPERATION_OUTPUT as QUERY_OPERATION_LOCK,
    QUERY_OUTPUT as QUERY_CONTRACT_LOCK,
    render_inventory as render_request_inventory,
    render_operations as render_query_operations,
    render_query as render_query_contracts,
)
from hetzner_openapi_contracts import (
    digest,
    operation_rows,
    parameter_rows,
    schema_rows,
)

ROOT = Path(__file__).resolve().parents[1]
OP_LOCK = ROOT / "docs" / "API_FINGERPRINTS.tsv"
SCHEMA_LOCK = ROOT / "docs" / "API_SCHEMA_FINGERPRINTS.tsv"
PARAMETER_LOCK = ROOT / "docs" / "API_PARAMETER_FINGERPRINTS.tsv"
MATRIX = ROOT / "docs" / "API_MATRIX.md"
SPEC_LOCK = ROOT / "docs" / "SPEC_LOCK.md"
RESPONSE_LOCK = (
    ROOT / "crates" / "cloud-sdk-hetzner" / "src" / "serde" / "response_operations.tsv"
)
CLOUD_MODEL_SCHEMA_LOCK = (
    ROOT
    / "crates"
    / "cloud-sdk-hetzner"
    / "src"
    / "serde"
    / "cloud_model_schema.tsv"
)
CLOUD_MODEL_FIXTURE_LOCK = CLOUD_MODEL_SCHEMA_LOCK.with_name(
    "cloud_model_fixtures.json"
)
PARAMETER_FIELDS = [
    "api",
    "method",
    "path",
    "operation_id",
    "in",
    "name",
    "required",
    "schema_type",
    "schema_format",
    "items_type",
    "style",
    "explode",
    "enum",
    "constraints",
    "fingerprint",
]

SPECS = {
    "cloud": "https://docs.hetzner.cloud/cloud.spec.json",
    "hetzner": "https://docs.hetzner.cloud/hetzner.spec.json",
}

PINNED_SPEC_SHA256 = {
    "cloud": "9ca6b542a057b002804b9f4f45ccfdb8b9a28c92b7e5bf5ae1b7f46b54fe0093",
    "hetzner": "f70750016d81c927ddf877e103541c90d3e3372723cdf54e6fd7b2eba4a8108a",
}

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
    for path in (
        OP_LOCK,
        SCHEMA_LOCK,
        PARAMETER_LOCK,
        MATRIX,
        SPEC_LOCK,
        RESPONSE_LOCK,
        CLOUD_MODEL_SCHEMA_LOCK,
        CLOUD_MODEL_FIXTURE_LOCK,
        QUERY_CONTRACT_LOCK,
        QUERY_OPERATION_LOCK,
        REQUEST_INVENTORY_LOCK,
    ):
        if not path.is_file() or path.stat().st_size == 0:
            print(f"missing required lock file: {path}", file=sys.stderr)
            status = 1
    if status:
        return status

    operation_count = len(read_tsv(OP_LOCK))
    schema_count = len(read_tsv(SCHEMA_LOCK))
    parameter_count = len(read_tsv(PARAMETER_LOCK))
    model_field_count = len(read_tsv(CLOUD_MODEL_SCHEMA_LOCK))
    parameter_rows_locked = read_tsv(PARAMETER_LOCK)
    generated_request_files = (
        (QUERY_CONTRACT_LOCK, render_query_contracts(parameter_rows_locked)),
        (QUERY_OPERATION_LOCK, render_query_operations(parameter_rows_locked)),
        (REQUEST_INVENTORY_LOCK, render_request_inventory(parameter_rows_locked)),
    )
    for path, expected in generated_request_files:
        if path.read_text(encoding="ascii") != expected:
            print(f"stale request contract inventory: {path}", file=sys.stderr)
            status = 1
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
    print(f"locked parameters: {parameter_count}")
    print(f"locked Cloud model fields: {model_field_count}")
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
    try:
        response_lock = render_response_operations(
            response_operation_rows("cloud", documents["cloud"])
            + response_operation_rows("hetzner", documents["hetzner"])
        )
        cloud_model_schema_lock = render_cloud_model_schema(
            documents["cloud"], documents["hetzner"]
        )
        cloud_model_fixture_lock = render_cloud_model_fixtures(
            documents["cloud"], documents["hetzner"]
        )
    except ValueError as error:
        raise SystemExit(f"invalid response schemas: {error}") from error
    operations = []
    schemas = []
    parameters = []
    for api, document in documents.items():
        operations.extend(operation_rows(api, document))
        schemas.extend(schema_rows(api, document))
        parameters.extend(parameter_rows(api, document))

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
                read_tsv(PARAMETER_LOCK),
                parameters,
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
        write_tsv(PARAMETER_LOCK, parameters, PARAMETER_FIELDS)
        RESPONSE_LOCK.write_text(response_lock, encoding="ascii")
        CLOUD_MODEL_SCHEMA_LOCK.write_text(cloud_model_schema_lock, encoding="ascii")
        CLOUD_MODEL_FIXTURE_LOCK.write_text(cloud_model_fixture_lock, encoding="ascii")
        print(f"wrote {len(operations)} operation fingerprints")
        print(f"wrote {len(schemas)} schema fingerprints")
        print(f"wrote {len(parameters)} parameter fingerprints")
        return 0

    status = validate_local_files()
    if RESPONSE_LOCK.read_text(encoding="ascii") != response_lock:
        print("Hetzner success-response operation lock has drift", file=sys.stderr)
        status = 1
    if CLOUD_MODEL_SCHEMA_LOCK.read_text(encoding="ascii") != cloud_model_schema_lock:
        print("Hetzner Cloud model schema lock has drift", file=sys.stderr)
        status = 1
    if CLOUD_MODEL_FIXTURE_LOCK.read_text(encoding="ascii") != cloud_model_fixture_lock:
        print("Hetzner Cloud model fixtures have drift", file=sys.stderr)
        status = 1
    report = build_drift_report(
        read_tsv(OP_LOCK),
        operations,
        read_tsv(SCHEMA_LOCK),
        schemas,
        source_hashes,
        PINNED_SPEC_SHA256,
        read_tsv(PARAMETER_LOCK),
        parameters,
    )
    status |= print_drift_report(report)
    return status


if __name__ == "__main__":
    raise SystemExit(main())
