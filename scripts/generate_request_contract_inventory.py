#!/usr/bin/env python3
"""Generate executable query contracts and the reviewed request inventory."""

from __future__ import annotations

import argparse
import csv
import io
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PARAMETERS = ROOT / "docs" / "API_PARAMETER_FINGERPRINTS.tsv"
OPERATIONS = ROOT / "docs" / "API_FINGERPRINTS.tsv"
BODIES = ROOT / "docs" / "PREPARED_BODY_OPERATIONS.txt"
QUERY_OUTPUT = (
    ROOT / "crates" / "cloud-sdk-hetzner" / "src" / "query" / "source_contracts.tsv"
)
OPERATION_OUTPUT = QUERY_OUTPUT.with_name("source_operations.rs")
INVENTORY_OUTPUT = ROOT / "docs" / "REQUEST_CONTRACT_INVENTORY.tsv"

QUERY_FIELDS = [
    "operation_id",
    "name",
    "required",
    "value_kind",
    "wire_encoding",
    "enum",
    "fingerprint",
]
INVENTORY_FIELDS = [
    "api",
    "method",
    "path",
    "operation_id",
    "location",
    "name",
    "required",
    "value_kind",
    "wire_encoding",
    "style",
    "explode",
    "enum",
    "constraints",
    "fingerprint",
    "implementation",
]


def read_parameters(path: Path = PARAMETERS) -> list[dict[str, str]]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


def deprecated_operations(path: Path = OPERATIONS) -> set[str]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        return {
            row["operation_id"]
            for row in csv.DictReader(handle, delimiter="\t")
            if row["deprecated"] == "yes"
        }


def enum_values(value: str) -> str:
    decoded = json.loads(value)
    if not isinstance(decoded, list) or not all(isinstance(item, str) for item in decoded):
        raise ValueError("parameter enum must be a string array")
    if any("|" in item or "\t" in item or "\n" in item for item in decoded):
        raise ValueError("parameter enum cannot be represented safely")
    return "|".join(decoded)


def value_kind(row: dict[str, str]) -> str:
    schema_type = row["schema_type"]
    if schema_type == "array":
        item = row["items_type"]
        if item not in {"integer", "string"}:
            raise ValueError(f"unsupported query array item type: {item}")
        return f"{item}[]"
    if schema_type not in {"boolean", "integer", "string"}:
        raise ValueError(f"unsupported query type: {schema_type}")
    return schema_type


def wire_encoding(row: dict[str, str]) -> str:
    if row["operation_id"] in {"get_server_metrics", "get_load_balancer_metrics"}:
        if row["name"] == "type":
            return "comma"
    return "repeat" if row["schema_type"] == "array" else "scalar"


def render_query(rows: list[dict[str, str]]) -> str:
    deprecated = deprecated_operations()
    output = io.StringIO()
    writer = csv.DictWriter(
        output, delimiter="\t", fieldnames=QUERY_FIELDS, lineterminator="\n"
    )
    writer.writeheader()
    for row in rows:
        if row["in"] != "query":
            continue
        if row["operation_id"] in deprecated:
            continue
        if row["style"] != "form" or row["explode"] != "yes":
            raise ValueError(
                f"unsupported query encoding for {row['operation_id']}:{row['name']}"
            )
        writer.writerow(
            {
                "operation_id": row["operation_id"],
                "name": row["name"],
                "required": row["required"],
                "value_kind": value_kind(row),
                "wire_encoding": wire_encoding(row),
                "enum": enum_values(row["enum"]),
                "fingerprint": row["fingerprint"],
            }
        )
    return output.getvalue()


def body_operations(path: Path = BODIES) -> list[str]:
    return [
        line
        for line in path.read_text(encoding="ascii").splitlines()
        if line and not line.startswith("#")
    ]


def render_inventory(rows: list[dict[str, str]]) -> str:
    deprecated = deprecated_operations()
    output = io.StringIO()
    writer = csv.DictWriter(
        output, delimiter="\t", fieldnames=INVENTORY_FIELDS, lineterminator="\n"
    )
    writer.writeheader()
    for row in rows:
        writer.writerow(
            {
                "api": row["api"],
                "method": row["method"],
                "path": row["path"],
                "operation_id": row["operation_id"],
                "location": row["in"],
                "name": row["name"],
                "required": row["required"],
                "value_kind": value_kind(row),
                "wire_encoding": wire_encoding(row),
                "style": row["style"],
                "explode": row["explode"],
                "enum": enum_values(row["enum"]),
                "constraints": row["constraints"],
                "fingerprint": row["fingerprint"],
                "implementation": "excluded-deprecated"
                if row["operation_id"] in deprecated
                else ("source-locked-query" if row["in"] == "query" else "typed-endpoint"),
            }
        )
    for operation in body_operations():
        writer.writerow(
            {
                "operation_id": operation,
                "location": "request-body",
                "required": "yes",
                "implementation": "typed-json-body",
            }
        )
    return output.getvalue()


def render_operations(rows: list[dict[str, str]]) -> str:
    deprecated = deprecated_operations()
    operations = sorted(
        {
            row["operation_id"]
            for row in rows
            if row["in"] == "query" and row["operation_id"] not in deprecated
        }
    )
    lines = [
        "// Generated by scripts/generate_request_contract_inventory.py.",
        "impl SourceQueryOperation {",
    ]
    for operation in operations:
        constant = operation.upper()
        lines.extend(
            [
                f"    /// Source-locked `{operation}` query operation.",
                f'    pub const {constant}: Self = Self("{operation}");',
            ]
        )
    lines.append("}")
    return "\n".join(lines) + "\n"


def check(path: Path, expected: str) -> bool:
    if not path.is_file() or path.read_text(encoding="ascii") != expected:
        print(f"request contract inventory is stale: {path.relative_to(ROOT)}")
        return False
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    try:
        rows = read_parameters()
        query = render_query(rows)
        inventory = render_inventory(rows)
        operations = render_operations(rows)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        raise SystemExit(f"request contract inventory failed: {error}") from error

    if args.write:
        QUERY_OUTPUT.parent.mkdir(parents=True, exist_ok=True)
        QUERY_OUTPUT.write_text(query, encoding="ascii")
        OPERATION_OUTPUT.write_text(operations, encoding="ascii")
        INVENTORY_OUTPUT.write_text(inventory, encoding="ascii")
        active_queries = sum(
            row["in"] == "query" and row["operation_id"] not in deprecated_operations()
            for row in rows
        )
        print(f"wrote {active_queries} active query contracts")
        print(f"wrote {len(rows) + len(body_operations())} request inventory rows")
        return 0

    status = (
        check(QUERY_OUTPUT, query)
        and check(OPERATION_OUTPUT, operations)
        and check(INVENTORY_OUTPUT, inventory)
    )
    if status:
        print("request contract inventory is current")
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
