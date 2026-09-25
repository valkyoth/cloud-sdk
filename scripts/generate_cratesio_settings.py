#!/usr/bin/env python3
"""Verify settings PATCH contracts and generate the bounded crate response schema."""
import argparse
import json
from pathlib import Path
from cratesio_source_fetch import fetch_source
from generate_cratesio_catalog import projection

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/cloud-sdk-cratesio/src/settings/schema_table.rs"
PATHS = {"update_crate": "/api/v1/crates/{name}",
         "update_version": "/api/v1/crates/{name}/{version}"}


def structural(value):
    if isinstance(value, dict):
        return {k: structural(v) for k, v in value.items()
                if k not in {"description", "example", "examples", "title"}}
    if isinstance(value, list):
        return [structural(v) for v in value]
    return value


def render(document):
    expected = {
        "update_crate": {"type": "object", "required": ["crate"], "properties": {
            "crate": {"oneOf": [{"type": "object", "properties": {
                "trustpub_only": {"type": ["boolean", "null"]}}}]}}},
        "update_version": {"type": "object", "required": ["version"], "properties": {
            "version": {"$ref": "#/components/schemas/VersionUpdate"}}},
    }
    if structural(document["components"]["schemas"]["VersionUpdate"]) != {
        "type": "object", "properties": {
            "yanked": {"type": ["boolean", "null"]},
            "yank_message": {"type": ["string", "null"]}}}:
        raise ValueError("version patch fields changed")
    roots = {}
    for name, path in PATHS.items():
        operation = document["paths"][path]["patch"]
        if operation["operationId"] != name:
            raise ValueError("settings operation identity changed")
        if operation["security"] != [{"api_token": []}, {"cookie": []}]:
            raise ValueError("settings authority changed")
        body = operation["requestBody"]
        if body.get("required") is not True or set(body["content"]) != {"application/json"}:
            raise ValueError("settings request media changed")
        if structural(body["content"]["application/json"]["schema"]) != expected[name]:
            raise ValueError("settings request fields changed")
        if {s for s in operation["responses"] if s.startswith("2")} != {"200"}:
            raise ValueError("settings success status changed")
        schema = operation["responses"]["200"]["content"]["application/json"]["schema"]
        field = "crate" if name == "update_crate" else "version"
        if structural(schema) != {"type": "object", "required": [field], "properties": {
            field: {"$ref": "#/components/schemas/" + field.capitalize()}}}:
            raise ValueError("settings success envelope changed")
        if field == "version":
            detail = document["paths"][path]["get"]["responses"]["200"]["content"]["application/json"]["schema"]
            if structural(detail) != structural(schema):
                raise ValueError("version detail decoder no longer matches PATCH response")
        else:
            roots[name.upper()] = schema
    return projection(document, roots).replace(
        "use super::schema::Node;", "use crate::catalog::schema::Node;"
    ).replace("generate_cratesio_catalog.py", "generate_cratesio_settings.py")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    lock = json.loads((ROOT / "provider-drift/providers/cratesio-source.lock.json").read_bytes())
    result = render(json.loads(fetch_source(next(s for s in lock["sources"] if s["id"] == "openapi"))))
    if args.write:
        OUTPUT.write_text(result, encoding="ascii")
    elif OUTPUT.read_text(encoding="ascii") != result:
        raise ValueError("settings projection is stale")
    print("Settings request, authority, response and status contracts passed.")


if __name__ == "__main__":
    main()
