#!/usr/bin/env python3
"""Generate the complete reviewed Hetzner typed-operation binding manifest."""

from __future__ import annotations

import argparse
import csv
import io
from pathlib import Path

import generate_operation_associations as associations

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "docs" / "TYPED_OPERATION_BINDINGS.tsv"

COLUMNS = (
    "operation_id",
    "api",
    "service",
    "method",
    "path",
    "endpoint_policy",
    "authentication",
    "authentication_scope",
    "query_policy",
    "body_policy",
    "request_headers",
    "request_media",
    "success_status",
    "success_shape",
    "success_root",
    "success_required",
    "success_body",
    "success_media",
    "success_body_max_bytes",
    "error_body",
    "error_media",
    "error_body_max_bytes",
    "pagination",
    "quota",
    "retry",
    "streaming",
    "permit_class",
    "response_identity",
)


def binding_row(operation: associations.Operation) -> dict[str, str]:
    """Return the complete public contract represented by one marker row."""
    has_body = operation.body_policy == "json"
    has_success_body = operation.response != "empty"
    return {
        "operation_id": operation.operation_id,
        "api": operation.api,
        "service": operation.service,
        "method": operation.method,
        "path": operation.path,
        "endpoint_policy": (
            "console-v1" if operation.service == "storage" else "cloud-v1"
        ),
        "authentication": operation.authentication,
        "authentication_scope": "required-service",
        "query_policy": operation.query_policy,
        "body_policy": operation.body_policy,
        "request_headers": (
            "accept-json+content-type-json" if has_body else "accept-json"
        ),
        "request_media": "application-json" if has_body else "forbidden",
        "success_status": operation.status,
        "success_shape": operation.response,
        "success_root": operation.response_root,
        "success_required": operation.response_required,
        "success_body": "required-json" if has_success_body else "forbidden",
        "success_media": "application-json" if has_success_body else "forbidden",
        "success_body_max_bytes": "8388608" if has_success_body else "0",
        "error_body": "required-json",
        "error_media": "application-json",
        "error_body_max_bytes": "8388608",
        "pagination": "numbered" if operation.pagination == "yes" else "none",
        "quota": "hetzner",
        "retry": operation.retry_policy,
        "streaming": "buffered",
        "permit_class": operation.permit_class,
        "response_identity": "explicit-endpoint",
    }


def binding_rows() -> list[dict[str, str]]:
    """Load all source locks and return rows in stable operation-ID order."""
    return [binding_row(operation) for operation in associations.load_operations()]


def render() -> str:
    """Render canonical ASCII TSV without platform-dependent newlines."""
    output = io.StringIO(newline="")
    writer = csv.DictWriter(output, fieldnames=COLUMNS, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    writer.writerows(binding_rows())
    return output.getvalue()


def read_manifest(path: Path = OUTPUT) -> list[dict[str, str]]:
    """Read a manifest while enforcing its exact schema and row shape."""
    with path.open(encoding="ascii", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if tuple(reader.fieldnames or ()) != COLUMNS:
            raise ValueError("typed operation binding manifest has an invalid schema")
        rows = list(reader)
    if any(None in row or any(value is None for value in row.values()) for row in rows):
        raise ValueError("typed operation binding manifest has a malformed row")
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    generated = render()
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text(encoding="ascii") != generated:
            raise SystemExit("typed operation binding manifest is stale; regenerate it")
        print(f"{associations.EXPECTED_OPERATIONS} typed operation bindings are current.")
        return 0
    OUTPUT.write_text(generated, encoding="ascii")
    print(f"generated {associations.EXPECTED_OPERATIONS} typed operation bindings")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
