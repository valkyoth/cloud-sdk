#!/usr/bin/env python3
"""Generate the source-locked Hetzner success-response operation table."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_OUTPUT = (
    ROOT / "crates" / "cloud-sdk-hetzner" / "src" / "serde" / "response_operations.tsv"
)
METHODS = ("get", "post", "put", "delete")
EXPECTED_ACTIVE = 208
SPECIAL_KEYS = {
    "action",
    "actions",
    "folders",
    "meta",
    "metrics",
    "next_actions",
    "password",
    "pricing",
    "root_password",
    "wss_url",
    "zonefile",
}


def load_spec(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read {path}: {error}") from error
    if not isinstance(value, dict) or not isinstance(value.get("paths"), dict):
        raise ValueError(f"{path} has no OpenAPI paths object")
    return value


def success_schema(operation: dict[str, Any]) -> tuple[str, dict[str, Any]]:
    responses = operation.get("responses")
    if not isinstance(responses, dict):
        raise ValueError("operation has no responses object")
    successes = [(key, value) for key, value in responses.items() if key.startswith("2")]
    if len(successes) != 1:
        raise ValueError("operation must have exactly one success response")
    status, response = successes[0]
    if not isinstance(response, dict):
        raise ValueError("success response is not an object")
    content = response.get("content", {})
    if not isinstance(content, dict):
        raise ValueError("success response content is not an object")
    media = content.get("application/json")
    if media is None:
        return status, {}
    if not isinstance(media, dict) or not isinstance(media.get("schema"), dict):
        raise ValueError("JSON success response has no schema object")
    return status, media["schema"]


def classify(schema: dict[str, Any]) -> tuple[str, str, str]:
    properties = schema.get("properties", {})
    if not isinstance(properties, dict):
        raise ValueError("success schema properties is not an object")
    keys = sorted(properties)
    required = schema.get("required", [])
    if not isinstance(required, list) or not all(isinstance(item, str) for item in required):
        raise ValueError("success schema required list is invalid")
    required_text = ",".join(sorted(required))
    resource_keys = [key for key in keys if key not in SPECIAL_KEYS]
    root = resource_keys[0] if len(resource_keys) == 1 else "-"
    if not keys:
        shape = "empty"
    elif keys == ["action"]:
        shape = "action"
    elif keys == ["actions", "meta"]:
        shape = "actions-page"
    elif keys == ["actions"]:
        shape = "actions"
    elif keys == ["metrics"]:
        shape = "metrics"
    elif keys == ["zonefile"]:
        shape = "zonefile"
    elif keys == ["pricing"]:
        shape = "pricing"
    elif keys == ["folders"]:
        shape = "folders"
    elif keys in (["snapshots"], ["subaccounts"]):
        shape = "resource-list"
    elif "meta" in keys and len(resource_keys) == 1:
        shape = "resource-page"
    elif len(keys) == 1 and len(resource_keys) == 1:
        shape = "resource"
    else:
        shape = "composite"
    if shape.startswith("resource") and root == "-":
        raise ValueError("resource response does not have exactly one resource key")
    return shape, root, required_text or "-"


def rows(api: str, spec: dict[str, Any]) -> list[tuple[str, ...]]:
    output: list[tuple[str, ...]] = []
    for path_value in spec["paths"].values():
        if not isinstance(path_value, dict):
            raise ValueError("path item is not an object")
        for method in METHODS:
            operation = path_value.get(method)
            if operation is None:
                continue
            if not isinstance(operation, dict):
                raise ValueError("operation is not an object")
            if operation.get("deprecated") is True:
                continue
            operation_id = operation.get("operationId")
            if not isinstance(operation_id, str) or not operation_id:
                raise ValueError("active operation has no operationId")
            validate_tsv_cell(operation_id, "operationId")
            status, schema = success_schema(operation)
            shape, root, required = classify(schema)
            output.append((api, operation_id, status, shape, root, required))
    return output


def render(all_rows: list[tuple[str, ...]]) -> str:
    if len(all_rows) != EXPECTED_ACTIVE:
        raise ValueError(f"expected {EXPECTED_ACTIVE} active operations, found {len(all_rows)}")
    operation_ids = [row[1] for row in all_rows]
    if len(set(operation_ids)) != len(operation_ids):
        raise ValueError("operation identifiers are not globally unique")
    for row in all_rows:
        for index, value in enumerate(row):
            validate_tsv_cell(value, f"response field {index}")
    lines = ["api\toperation_id\tstatus\tshape\troot\trequired"]
    lines.extend("\t".join(row) for row in sorted(all_rows, key=lambda row: row[1]))
    return "\n".join(lines) + "\n"


def validate_tsv_cell(value: str, field: str) -> None:
    if any(character in value for character in ("\t", "\n", "\r", "\0")):
        raise ValueError(f"{field} contains a TSV-unsafe character: {value!r}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("cloud_spec", type=Path)
    parser.add_argument("hetzner_spec", type=Path)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    try:
        generated = render(
            rows("cloud", load_spec(args.cloud_spec))
            + rows("hetzner", load_spec(args.hetzner_spec))
        )
        if args.check:
            current = args.output.read_text(encoding="ascii")
            if current != generated:
                raise ValueError("committed response-operation table is stale")
        else:
            args.output.write_text(generated, encoding="ascii")
    except (OSError, UnicodeError, ValueError) as error:
        print(f"response operation generation: {error}", file=sys.stderr)
        return 1
    print(f"response operation generation: {EXPECTED_ACTIVE} active operations checked")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
