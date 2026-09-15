#!/usr/bin/env python3
"""Check version schema projections and source-derived JSON fixtures."""
import argparse
import json
from pathlib import Path
from cratesio_source_fetch import fetch_source
from generate_cratesio_catalog import expanded_example, projection
from generate_cratesio_discovery_fixtures import verify

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/cloud-sdk-cratesio/src/versions"
PATHS = {
    "list_versions": "/api/v1/crates/{name}/versions",
    "find_version": "/api/v1/crates/{name}/{version}",
    "get_version_dependencies": "/api/v1/crates/{name}/{version}/dependencies",
    "get_version_readme": "/api/v1/crates/{name}/{version}/readme",
}


def render(document):
    roots, fixtures = {}, {}
    for name, path in PATHS.items():
        op = document["paths"][path]["get"]
        if op["operationId"] != name:
            raise ValueError("version operation identity changed")
        schema = op["responses"]["200"]["content"]["application/json"]["schema"]
        roots[name.upper()] = schema
    table = projection(document, roots)
    for name in PATHS:
        schema = roots[name.upper()]
        value = expanded_example(schema, document)
        if name == "list_versions":
            value["meta"] = {"total": 1, "next_page": None}
        fixtures[name + ".json"] = json.dumps(value, sort_keys=True, indent=2) + "\n"
    table = table.replace("use super::schema::Node;", "use crate::catalog::schema::Node;")
    table = table.replace("generate_cratesio_catalog.py", "generate_cratesio_versions.py")
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
            raise ValueError("version projection is stale")
        verify(fixtures, OUTPUT / "fixtures")
    print("4 version JSON projections and fixtures passed; authors use the pinned controller contract.")


if __name__ == "__main__":
    main()
