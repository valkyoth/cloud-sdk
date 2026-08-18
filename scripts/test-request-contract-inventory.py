#!/usr/bin/env python3
"""Regression tests for request-contract inventory generation."""

from __future__ import annotations

import copy
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = ROOT / "scripts" / "generate_request_contract_inventory.py"
SPEC = importlib.util.spec_from_file_location("request_inventory", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def expect_failure(rows: list[dict[str, str]], message: str) -> None:
    try:
        MODULE.render_query(rows)
    except ValueError as error:
        assert message in str(error), error
    else:
        raise AssertionError("invalid request contract was accepted")


def main() -> None:
    rows = MODULE.read_parameters()
    query = MODULE.render_query(rows)
    inventory = MODULE.render_inventory(rows)
    operations = MODULE.render_operations(rows)
    assert query.count("\n") == 219
    assert "get_server_metrics\ttype\tyes\tstring[]\tcomma\tcpu|disk|network\t" in query
    assert "list_servers\tsort\tno\tstring[]\trepeat\t" in query
    assert inventory.count("\n") == 1 + len(rows) + len(MODULE.body_operations())
    assert "\trequest-body\t" in inventory
    assert "\ttyped-json-body\n" in inventory
    assert operations.count("pub const ") == 47
    assert 'pub const GET_SERVER_METRICS: Self = Self("get_server_metrics");' in operations
    assert "LIST_DATACENTERS" not in operations
    assert "list_datacenters\tquery\tname\tno\tstring" in inventory
    assert "excluded-deprecated" in inventory

    changed = copy.deepcopy(rows)
    query_row = next(row for row in changed if row["in"] == "query")
    query_row["style"] = "spaceDelimited"
    expect_failure(changed, "unsupported query encoding")

    changed = copy.deepcopy(rows)
    query_row = next(row for row in changed if row["in"] == "query")
    query_row["schema_type"] = "object"
    expect_failure(changed, "unsupported query type")

    baseline = MODULE.render_query(rows)
    changed = copy.deepcopy(rows)
    query_row = next(row for row in changed if row["in"] == "query")
    query_row["required"] = "no" if query_row["required"] == "yes" else "yes"
    assert MODULE.render_query(changed) != baseline

    changed = copy.deepcopy(rows)
    query_row = next(
        row for row in changed if row["in"] == "query" and row["schema_type"] == "string"
    )
    query_row["schema_type"] = "array"
    query_row["items_type"] = "string"
    assert MODULE.render_query(changed) != baseline

    changed = copy.deepcopy(rows)
    query_row = next(
        row for row in changed if row["in"] == "query" and row["enum"] != "[]"
    )
    query_row["enum"] = '["source-mutation"]'
    assert "source-mutation" in MODULE.render_query(changed)

    changed = copy.deepcopy(rows)
    query_row = next(row for row in changed if row["in"] == "query")
    query_row["fingerprint"] = "0" * 64
    assert MODULE.render_query(changed) != baseline

    changed = copy.deepcopy(rows)
    source = next(row for row in changed if row["in"] == "query")
    added = copy.deepcopy(source)
    added["name"] = "future_parameter"
    added["fingerprint"] = "1" * 64
    changed.append(added)
    assert "future_parameter" in MODULE.render_query(changed)

    print("10 request contract inventory regression groups passed.")


if __name__ == "__main__":
    main()
