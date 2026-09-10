#!/usr/bin/env python3
"""Generate bounded discovery fixtures from the exact admitted OpenAPI source."""
import argparse
import json
from pathlib import Path
from cratesio_source_fetch import fetch_source

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/cloud-sdk-cratesio/src/discovery/fixtures"
PATHS = (
    "/api/v1/categories", "/api/v1/categories/{category}", "/api/v1/category_slugs",
    "/api/v1/keywords", "/api/v1/keywords/{keyword}", "/api/v1/site_metadata", "/api/v1/summary",
)
OPERATIONS = ("list_categories", "find_category", "list_category_slugs",
              "list_keywords", "find_keyword", "get_site_metadata", "get_summary")


def example(schema, document, depth=0):
    if depth > 16:
        raise ValueError("fixture reference nesting exceeded")
    if "$ref" in schema:
        prefix = "#/components/schemas/"
        reference = schema["$ref"]
        if not reference.startswith(prefix):
            raise ValueError("unsupported fixture reference")
        return example(document["components"]["schemas"][reference[len(prefix):]], document, depth+1)
    if "example" in schema:
        return schema["example"]
    kind = schema.get("type")
    if isinstance(kind, list) and "null" in kind:
        return None
    if kind == "object":
        return {key: example(schema["properties"][key], document, depth+1)
                for key in schema.get("required", [])}
    if kind == "array":
        return [example(schema["items"], document, depth+1)]
    if kind == "string":
        if schema.get("format") == "date-time":
            return "2026-09-10T00:00:00Z"
        return "fixture"
    if kind == "integer":
        return max(1, schema.get("minimum", 0))
    if kind == "boolean":
        return False
    raise ValueError("unreviewed fixture schema")


def render(document):
    result = {}
    for path, name in zip(PATHS, OPERATIONS, strict=True):
        operation = document["paths"][path]["get"]
        if operation["operationId"] != name:
            raise ValueError("discovery operation identity changed")
        schema = operation["responses"]["200"]["content"]["application/json"]["schema"]
        result[name + ".json"] = json.dumps(
            example(schema, document), sort_keys=True, indent=2, ensure_ascii=True) + "\n"
    return result


def verify(fixtures, output):
    if {p.name for p in output.glob("*.json")} != set(fixtures):
        raise ValueError("discovery fixture inventory differs")
    for name, content in fixtures.items():
        if (output / name).read_text(encoding="ascii") != content:
            raise ValueError(f"stale discovery fixture: {name}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    lock = json.loads((ROOT / "provider-drift/providers/cratesio-source.lock.json").read_bytes())
    source = next(s for s in lock["sources"] if s["id"] == "openapi")
    fixtures = render(json.loads(fetch_source(source)))
    if args.write:
        OUTPUT.mkdir(parents=True, exist_ok=True)
        for name, content in fixtures.items():
            (OUTPUT / name).write_text(content, encoding="ascii")
    else:
        verify(fixtures, OUTPUT)
    print("7 source-derived crates.io discovery fixtures passed.")


if __name__ == "__main__":
    main()
