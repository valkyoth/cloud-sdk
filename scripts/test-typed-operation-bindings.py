#!/usr/bin/env python3
"""Regression tests for the complete typed-operation binding manifest."""

from __future__ import annotations

import csv
import os
import subprocess
import sys
import tempfile
from pathlib import Path

import check_typed_operation_bindings as checker
import generate_typed_operation_bindings as bindings

if sys.flags.optimize:
    raise SystemExit("security regression tests must not run with Python optimization")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> int:
    rows = bindings.binding_rows()
    require(len(rows) == 208, "expected exactly 208 binding rows")
    require(
        [row["operation_id"] for row in rows]
        == sorted(row["operation_id"] for row in rows),
        "binding rows are not sorted",
    )
    require(
        all(tuple(row) == bindings.COLUMNS for row in rows),
        "binding row schema changed",
    )
    by_id = {row["operation_id"]: row for row in rows}
    require(by_id["list_servers"]["path"] == "/servers", "server path changed")
    require(by_id["list_servers"]["query_policy"] == "optional", "query changed")
    require(by_id["create_server"]["body_policy"] == "json", "body changed")
    require(by_id["create_server"]["permit_class"] == "cost", "permit changed")
    require(by_id["delete_ssh_key"]["success_body"] == "forbidden", "body changed")
    require(
        by_id["list_storage_boxes"]["endpoint_policy"] == "console-v1",
        "endpoint changed",
    )
    require(
        by_id["list_storage_boxes"]["success_root"] == "storage_boxes",
        "response root changed",
    )
    require(
        by_id["get_storage_box"]["response_identity"] == "exact-resource",
        "exact response identity changed",
    )
    require(
        by_id["list_storage_box_snapshots"]["response_identity"] == "parent-resource",
        "parent response identity changed",
    )
    require(
        by_id["list_servers"]["response_identity"] == "none",
        "unmodeled response identity changed",
    )

    deprecated = {
        row["operation_id"]
        for row in bindings.associations.read_tsv(bindings.associations.FINGERPRINTS)
        if row["deprecated"] == "yes"
    }
    require(len(deprecated) == 13, "deprecated count changed")
    require(not deprecated & set(by_id), "deprecated binding admitted")
    require(len(checker.EXPECTED_REJECTING_BODY_VARIANTS) == 12, "body set changed")
    require(
        all(
            by_id[operation]["body_policy"] == "forbidden"
            for operation in checker.EXPECTED_REJECTING_BODY_VARIANTS
        ),
        "rejecting body variant became body-bearing",
    )

    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "bindings.tsv"
        path.write_text(bindings.render(), encoding="ascii")
        require(bindings.read_manifest(path) == rows, "manifest round trip changed")
        with path.open("w", encoding="ascii", newline="") as handle:
            writer = csv.DictWriter(
                handle,
                fieldnames=bindings.COLUMNS[:-1],
                delimiter="\t",
                lineterminator="\n",
            )
            writer.writeheader()
        try:
            bindings.read_manifest(path)
        except ValueError as error:
            require("invalid schema" in str(error), "wrong schema failure")
        else:
            raise AssertionError("a truncated binding schema was accepted")

    optimized = subprocess.run(
        [sys.executable, "-O", str(Path(__file__).resolve())],
        cwd=bindings.ROOT,
        env={**os.environ, "PYTHONOPTIMIZE": ""},
        capture_output=True,
        text=True,
        check=False,
    )
    require(optimized.returncode != 0, "optimized execution was accepted")
    require("must not run" in optimized.stderr, "optimized rejection was not explicit")

    print("208 complete typed bindings and 13 deprecated exclusions tested.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
