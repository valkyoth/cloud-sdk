#!/usr/bin/env python3
"""Check account schema projections and source-derived JSON fixtures."""
import argparse
import copy
import json
from pathlib import Path
from cratesio_source_fetch import fetch_source
from generate_cratesio_catalog import expanded_example, projection
from generate_cratesio_discovery_fixtures import verify

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/cloud-sdk-cratesio/src/accounts"
PATHS = {
    "find_user": "/api/v1/users/{user}",
    "get_user_stats": "/api/v1/users/{id}/stats",
    "find_team": "/api/v1/teams/{team}",
    "list_owners": "/api/v1/crates/{name}/owners",
    "get_user_owners": "/api/v1/crates/{name}/owner_user",
    "get_team_owners": "/api/v1/crates/{name}/owner_team",
}


def render(document):
    roots, fixtures = {}, {}
    for name, path in PATHS.items():
        op = document["paths"][path]["get"]
        if op["operationId"] != name:
            raise ValueError("account operation identity changed")
        schema = op["responses"]["200"]["content"]["application/json"]["schema"]
        schema = copy.deepcopy(schema)
        if name == "get_user_stats":
            count = schema["properties"]["total_downloads"]
            if count.get("type") != "integer" or count.get("format") != "int64" or type(count.get("minimum")) is not int or count["minimum"] != 0:
                raise ValueError("unreviewed user statistics bound")
            # Integer nodes already enforce a nonnegative signed-64-bit bound.
            del count["minimum"]
        roots[name.upper()] = schema
    table = projection(document, roots)
    for name in PATHS:
        schema = roots[name.upper()]
        value = expanded_example(schema, document)
        if name == "find_user":
            value.pop("linked_accounts", None)
        if name == "get_team_owners":
            owner = document["components"]["schemas"]["Owner"]["oneOf"][1]
            value["teams"] = [expanded_example(owner, document)]
            value["teams"][0]["kind"] = owner["properties"]["kind"]["enum"][0]
        elif name in ("list_owners", "get_user_owners"):
            owner = document["components"]["schemas"]["Owner"]["oneOf"][0]
            value["users"][0]["kind"] = owner["properties"]["kind"]["enum"][0]
        fixtures[name + ".json"] = json.dumps(value, sort_keys=True, indent=2) + "\n"
    table = table.replace("use super::schema::Node;", "use crate::catalog::schema::Node;")
    table = table.replace("generate_cratesio_catalog.py", "generate_cratesio_accounts.py")
    return table, fixtures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    lock = json.loads((ROOT / "provider-drift/providers/cratesio-source.lock.json").read_bytes())
    source = next(s for s in lock["sources"] if s["id"] == "openapi")
    table, fixtures = render(json.loads(fetch_source(source)))
    if args.write:
        (OUTPUT / "fixtures").mkdir(parents=True, exist_ok=True)
        (OUTPUT / "schema_table.rs").write_text(table, encoding="ascii")
        for name, value in fixtures.items():
            (OUTPUT / "fixtures" / name).write_text(value, encoding="ascii")
    else:
        if (OUTPUT / "schema_table.rs").read_text(encoding="ascii") != table:
            raise ValueError("account projection is stale")
        verify(fixtures, OUTPUT / "fixtures")
    print("6 account JSON projections and fixtures passed.")


if __name__ == "__main__":
    main()
