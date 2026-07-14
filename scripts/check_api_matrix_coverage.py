#!/usr/bin/env python3
"""Require every non-deprecated API-matrix operation to be implemented."""

from __future__ import annotations

import argparse
import re
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_MATRIX = ROOT / "docs" / "API_MATRIX.md"
EXPECTED_TOTAL = 221
EXPECTED_NON_DEPRECATED = 208
EXPECTED_DEPRECATED = 13
MAX_MATRIX_BYTES = 2 * 1024 * 1024
IMPLEMENTED_STATUS = re.compile(r"implemented(?:-v[0-9]+\.[0-9]+)?")
EXPECTED_COLUMNS = (
    "API",
    "Group",
    "Method",
    "Path",
    "Operation",
    "Owner",
    "Pagination",
    "Sorting",
    "Action",
    "Deprecated",
    "Status",
)


@dataclass(frozen=True)
class Operation:
    """One validated operation row from the API matrix."""

    api: str
    method: str
    path: str
    operation: str
    deprecated: str
    status: str


def cells(line: str) -> tuple[str, ...]:
    """Split one Markdown table row into trimmed cells."""
    return tuple(cell.strip().strip("`") for cell in line.strip().strip("|").split("|"))


def parse_operations(path: Path) -> list[Operation]:
    """Parse and validate the operations table."""
    with path.open("rb") as matrix_file:
        payload = matrix_file.read(MAX_MATRIX_BYTES + 1)
    if len(payload) > MAX_MATRIX_BYTES:
        raise ValueError("matrix exceeds the local size limit")
    lines = payload.decode("utf-8").splitlines()
    try:
        start = lines.index("## Operations")
    except ValueError as error:
        raise ValueError("missing Operations section") from error

    cursor = start + 1
    while cursor < len(lines) and not lines[cursor].startswith("|"):
        cursor += 1
    table: list[str] = []
    while cursor < len(lines) and lines[cursor].startswith("|"):
        table.append(lines[cursor])
        cursor += 1
    if len(table) < 3:
        raise ValueError("operations table is missing or empty")
    header = cells(table[0])
    if header != EXPECTED_COLUMNS:
        raise ValueError("operations table columns changed unexpectedly")

    operations: list[Operation] = []
    for line_number, line in enumerate(table[2:], start=start + 4):
        row = cells(line)
        if len(row) != len(EXPECTED_COLUMNS):
            raise ValueError(f"malformed operation row at line {line_number}")
        values = dict(zip(EXPECTED_COLUMNS, row, strict=True))
        deprecated = values["Deprecated"]
        if deprecated not in {"yes", "no"}:
            raise ValueError(f"invalid deprecated value at line {line_number}")
        operations.append(
            Operation(
                api=values["API"],
                method=values["Method"],
                path=values["Path"],
                operation=values["Operation"],
                deprecated=deprecated,
                status=values["Status"],
            )
        )
    if not operations:
        raise ValueError("operations table contains no operation rows")
    return operations


def incomplete_non_deprecated(operations: list[Operation]) -> list[Operation]:
    """Return non-deprecated rows without an implemented status."""
    return [
        operation
        for operation in operations
        if operation.deprecated == "no"
        and IMPLEMENTED_STATUS.fullmatch(operation.status) is None
    ]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("matrix", nargs="?", type=Path, default=DEFAULT_MATRIX)
    parser.add_argument("--expected-total", type=int)
    parser.add_argument("--expected-non-deprecated", type=int)
    parser.add_argument("--expected-deprecated", type=int)
    args = parser.parse_args()

    try:
        operations = parse_operations(args.matrix)
    except (OSError, UnicodeError, ValueError) as error:
        raise SystemExit(f"API matrix coverage: {error}") from error

    incomplete = incomplete_non_deprecated(operations)
    if incomplete:
        for operation in incomplete:
            print(
                "API matrix coverage: "
                f"{operation.api} {operation.method} {operation.path} "
                f"({operation.operation}) is {operation.status}"
            )
        raise SystemExit("API matrix coverage: non-deprecated operations remain incomplete")

    covered = sum(operation.deprecated == "no" for operation in operations)
    deferred = len(operations) - covered
    expected = (
        args.expected_total,
        args.expected_non_deprecated,
        args.expected_deprecated,
    )
    if args.matrix.resolve() == DEFAULT_MATRIX.resolve() and expected == (None, None, None):
        expected = (EXPECTED_TOTAL, EXPECTED_NON_DEPRECATED, EXPECTED_DEPRECATED)
    if any(value is not None for value in expected) and any(
        value is None for value in expected
    ):
        raise SystemExit("API matrix coverage: all expected counts must be supplied together")
    if expected != (None, None, None) and (len(operations), covered, deferred) != expected:
        raise SystemExit(
            "API matrix coverage: source-locked operation counts changed unexpectedly"
        )
    print(
        "API matrix coverage: "
        f"{covered} non-deprecated operations implemented; "
        f"{deferred} deprecated operations deferred"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
