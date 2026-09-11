#!/usr/bin/env python3
"""Verify pinned catalog fixtures and the included-version schema projection."""
import argparse
import json
from pathlib import Path
from cratesio_source_fetch import fetch_source
from generate_cratesio_discovery_fixtures import example, verify

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/cloud-sdk-cratesio/src/catalog"
PATHS = {"list_crates": "/api/v1/crates", "find_crate": "/api/v1/crates/{name}",
         "find_new_crate": "/api/v1/crates/new"}

def expanded_example(schema, document, depth=0):
    if depth > 24:
        raise ValueError("catalog example nesting")
    if "$ref" in schema:
        reference = schema["$ref"]
        prefix = "#/components/schemas/"
        if not reference.startswith(prefix):
            raise ValueError("catalog example external reference")
        return expanded_example(document["components"]["schemas"][reference[len(prefix):]], document, depth+1)
    if "example" in schema:
        return schema["example"]
    if "oneOf" in schema:
        return expanded_example(schema["oneOf"][0], document, depth+1)
    if schema.get("type") == "null":
        return None
    if schema.get("type") == "object":
        return {k: expanded_example(schema["properties"][k], document, depth+1) for k in schema.get("required", [])}
    if schema.get("type") == "array":
        return [expanded_example(schema["items"], document, depth+1)]
    return example(schema, document, depth)


def render(document):
    schemas = document["components"]["schemas"]
    nodes = []
    def node(schema, depth=0):
        if depth > 24:
            raise ValueError("catalog schema depth/cycle")
        allowed = {"type", "$ref", "description", "deprecated", "example", "format", "properties", "required", "additionalProperties", "propertyNames", "items", "oneOf", "enum"}
        if set(schema) - allowed:
            raise ValueError("unreviewed catalog schema keyword")
        if schema.get("propertyNames", {"type": "string"}) != {"type": "string"}:
            raise ValueError("unreviewed catalog property-name constraint")
        if "format" in schema and schema["format"] not in ("date-time", "int32", "int64"):
            raise ValueError("unreviewed catalog format")
        def child(s):
            return node(s, depth + 1)
        if "$ref" in schema:
            if set(schema) - {"$ref", "description", "deprecated", "example"}:
                raise ValueError("catalog reference sibling constraint")
            prefix = "#/components/schemas/"
            if not schema["$ref"].startswith(prefix):
                raise ValueError("catalog external reference")
            return child(schemas[schema["$ref"][len(prefix):]])
        for forbidden in ("allOf", "anyOf", "not", "if", "then", "else", "pattern"):
            if forbidden in schema:
                raise ValueError(f"unreviewed catalog constraint: {forbidden}")
        kind = schema.get("type")
        if "oneOf" in schema:
            if set(schema) - {"oneOf", "description", "deprecated", "example"}:
                raise ValueError("catalog oneOf sibling constraint")
            choices = schema["oneOf"]
            if not isinstance(choices, list) or not choices or len(choices) > 8:
                raise ValueError("catalog oneOf branch count")
            entry = "Node::OneOf(&[" + ",".join(str(child(s)) for s in choices) + "])"
        elif isinstance(kind, list):
            if len(kind) != 2 or "null" not in kind:
                raise ValueError("catalog type union")
            entry = f"Node::Nullable({child(dict(schema, type=next(k for k in kind if k != 'null')))})"
        elif "enum" in schema:
            if kind != "string" or not all(isinstance(v, str) for v in schema["enum"]):
                raise ValueError("catalog enum type")
            entry = "Node::Enum(&[" + ",".join(json.dumps(s) for s in schema["enum"]) + "])"
        elif kind == "object":
            fields = schema.get("properties", {})
            required = schema.get("required", [])
            if not set(required) <= set(fields):
                raise ValueError("catalog missing required schema")
            entries = [f"({json.dumps(k)}, {child(v)}, {str(k in required).lower()})" for k, v in sorted(fields.items())]
            additional = schema.get("additionalProperties", {})
            if additional is False:
                raise ValueError("closed catalog object requires review")
            extra = f"Some({child({} if additional is True else additional)})"
            entry = "Node::Object(&[" + ",".join(entries) + f"], {extra})"
        elif kind == "array":
            entry = f"Node::Array({child(schema['items'])})"
        elif kind == "string":
            entry = "Node::Timestamp" if schema.get("format") == "date-time" else "Node::Text"
        elif kind == "integer":
            entry = "Node::Integer(" + ("2147483647" if schema.get("format") == "int32" else "9223372036854775807") + ")"
        elif kind in ("boolean", "null"):
            entry = "Node::Bool" if kind == "boolean" else "Node::Null"
        elif not schema:
            entry = "Node::Any"
        else:
            raise ValueError(f"unreviewed catalog schema: {schema}")
        nodes.append(entry)
        return len(nodes) - 1
    version = node(schemas["Version"])
    table = "// Generated by scripts/generate_cratesio_catalog.py; source review required.\nuse super::schema::Node;\n"
    table += f"pub(super) const VERSION: usize = {version};\n#[rustfmt::skip]\npub(super) const NODES: &[Node] = &[\n"
    table += "\n".join("    " + n + "," for n in nodes) + "\n];\n"
    fixtures = {}
    for name, path in PATHS.items():
        op = document["paths"][path]["get"]
        if op["operationId"] != name:
            raise ValueError("catalog operation identity changed")
        value = example(op["responses"]["200"]["content"]["application/json"]["schema"], document)
        if name == "list_crates":
            value["meta"] = {"total": 1, "next_page": None, "prev_page": None}
        elif name == "find_new_crate":
            value["crate"]["id"] = value["crate"]["name"] = "new"
        if name != "list_crates":
            # OpenAPI examples are nullable fragments, not a full include response.
            # Build a coherent full-include fixture from the same field schemas.
            version = expanded_example(schemas["Version"], document)
            version["crate"] = value["crate"]["name"]
            value["versions"] = [version]
            value["keywords"] = [expanded_example(schemas["Keyword"], document)]
            value["categories"] = [expanded_example(schemas["Category"], document)]
            value["crate"]["versions"] = [version["id"]]
            value["crate"]["default_version"] = version["num"]
            value["crate"]["keywords"] = [v["keyword"] for v in value["keywords"]]
            value["crate"]["categories"] = [v["slug"] for v in value["categories"]]
        fixtures[name + ".json"] = json.dumps(value, sort_keys=True, indent=2) + "\n"
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
            raise ValueError("catalog schema projection is stale")
        verify(fixtures, OUTPUT / "fixtures")
    print("3 catalog operation fixtures and included-version schema passed.")


if __name__ == "__main__":
    main()
