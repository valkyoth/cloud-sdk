#!/usr/bin/env python3
"""Verify one checked response binding for every active Hetzner operation."""

from __future__ import annotations

import csv
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parent.parent
MATRIX = ROOT / "docs" / "API_MATRIX.md"
LOCK = (
    ROOT / "crates" / "cloud-sdk-hetzner" / "src" / "serde" / "response_operations.tsv"
)
EXPECTED_ACTIVE = 208
SHAPES = {
    "empty",
    "action",
    "actions",
    "actions-page",
    "resource",
    "resource-list",
    "resource-page",
    "composite",
    "metrics",
    "zonefile",
    "pricing",
    "folders",
}


def markdown_code(value: str, field: str) -> str:
    if len(value) < 3 or not value.startswith("`") or not value.endswith("`"):
        raise ValueError(f"API matrix {field} is not canonical inline code")
    inner = value[1:-1]
    if "`" in inner or not inner:
        raise ValueError(f"API matrix {field} is malformed")
    return inner


def active_operations(text: str) -> set[str]:
    operations: set[str] = set()
    for line in text.splitlines():
        if not line.startswith("| cloud |") and not line.startswith("| hetzner |"):
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if len(cells) != 11:
            raise ValueError("API matrix operation row has the wrong column count")
        operation = markdown_code(cells[4], "operation")
        deprecated = cells[9]
        status = cells[10]
        if deprecated == "no":
            if not status.startswith("implemented"):
                raise ValueError(f"active operation is not implemented: {operation}")
            if operation in operations:
                raise ValueError(f"duplicate active operation: {operation}")
            operations.add(operation)
    return operations


def response_operations(path: Path) -> set[str]:
    with path.open("r", encoding="ascii", newline="") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    operations: set[str] = set()
    for row in rows:
        operation = row.get("operation_id", "")
        shape = row.get("shape", "")
        root = row.get("root", "")
        status = row.get("status", "")
        if not operation or operation in operations:
            raise ValueError("response operation identifiers are missing or duplicated")
        if shape not in SHAPES:
            raise ValueError(f"unknown response shape for {operation}")
        if shape.startswith("resource") and root == "-":
            raise ValueError(f"resource response has no root for {operation}")
        if status not in {"200", "201", "204"}:
            raise ValueError(f"unexpected success status for {operation}")
        operations.add(operation)
    return operations


def main() -> int:
    try:
        active = active_operations(MATRIX.read_text(encoding="utf-8"))
        responses = response_operations(LOCK)
        if len(active) != EXPECTED_ACTIVE or len(responses) != EXPECTED_ACTIVE:
            raise ValueError(
                f"expected {EXPECTED_ACTIVE} active/response operations, "
                f"found {len(active)}/{len(responses)}"
            )
        missing = sorted(active - responses)
        extra = sorted(responses - active)
        if missing or extra:
            raise ValueError(f"response coverage mismatch: missing={missing}, extra={extra}")
    except (OSError, UnicodeError, ValueError) as error:
        print(f"response operation coverage: {error}", file=sys.stderr)
        return 1
    print(f"Response operation coverage: {EXPECTED_ACTIVE} active operations checked.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
