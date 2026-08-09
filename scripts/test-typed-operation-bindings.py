#!/usr/bin/env python3
"""Regression tests for the complete typed-operation binding manifest."""

from __future__ import annotations

import csv
import tempfile
from pathlib import Path

import check_typed_operation_bindings as checker
import generate_typed_operation_bindings as bindings


def main() -> int:
    rows = bindings.binding_rows()
    assert len(rows) == 208
    assert [row["operation_id"] for row in rows] == sorted(
        row["operation_id"] for row in rows
    )
    assert all(tuple(row) == bindings.COLUMNS for row in rows)
    by_id = {row["operation_id"]: row for row in rows}
    assert by_id["list_servers"]["path"] == "/servers"
    assert by_id["list_servers"]["query_policy"] == "optional"
    assert by_id["create_server"]["body_policy"] == "json"
    assert by_id["create_server"]["permit_class"] == "cost"
    assert by_id["delete_ssh_key"]["success_body"] == "forbidden"
    assert by_id["list_storage_boxes"]["endpoint_policy"] == "console-v1"
    assert by_id["list_storage_boxes"]["success_root"] == "storage_boxes"
    assert all(row["response_identity"] == "explicit-endpoint" for row in rows)

    deprecated = {
        row["operation_id"]
        for row in bindings.associations.read_tsv(bindings.associations.FINGERPRINTS)
        if row["deprecated"] == "yes"
    }
    assert len(deprecated) == 13
    assert not deprecated & set(by_id)
    assert len(checker.EXPECTED_REJECTING_BODY_VARIANTS) == 12
    assert all(
        by_id[operation]["body_policy"] == "forbidden"
        for operation in checker.EXPECTED_REJECTING_BODY_VARIANTS
    )

    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "bindings.tsv"
        path.write_text(bindings.render(), encoding="ascii")
        assert bindings.read_manifest(path) == rows
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
            assert "invalid schema" in str(error)
        else:
            raise AssertionError("a truncated binding schema was accepted")

    print("208 complete typed bindings and 13 deprecated exclusions tested.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
