#!/usr/bin/env python3
"""Generate the source-locked schema table for ordinary Hetzner Cloud models."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUTPUT = (
    ROOT
    / "crates"
    / "cloud-sdk-hetzner"
    / "src"
    / "serde"
    / "cloud_model_schema.tsv"
)
DEFAULT_FIXTURES = (
    ROOT
    / "crates"
    / "cloud-sdk-hetzner"
    / "src"
    / "serde"
    / "cloud_model_fixtures.json"
)
METHODS = ("get", "post", "put", "delete")
MODEL_ROOTS = {
    "firewall": "firewall",
    "firewalls": "firewall",
    "floating_ip": "floating_ip",
    "floating_ips": "floating_ip",
    "image": "image",
    "images": "image",
    "iso": "iso",
    "isos": "iso",
    "load_balancer": "load_balancer",
    "load_balancers": "load_balancer",
    "load_balancer_type": "load_balancer_type",
    "load_balancer_types": "load_balancer_type",
    "location": "location",
    "locations": "location",
    "network": "network",
    "networks": "network",
    "placement_group": "placement_group",
    "placement_groups": "placement_group",
    "pricing": "pricing",
    "primary_ip": "primary_ip",
    "primary_ips": "primary_ip",
    "server": "server",
    "servers": "server",
    "server_type": "server_type",
    "server_types": "server_type",
    "volume": "volume",
    "volumes": "volume",
}
EXPECTED_MODELS = frozenset(MODEL_ROOTS.values())
SUPPORTED_FORMATS = frozenset(("date-time", "decimal", "double", "int32", "int64"))
SUPPORTED_PATTERNS = frozenset(
    (
        r"^[a-z0-9]+(-?[a-z0-9]*)*$",
        r"^\S(.*\S)?$",
    )
)
UNSUPPORTED_SECURITY_CONSTRAINTS = frozenset(
    (
        "exclusiveMinimum",
        "exclusiveMaximum",
        "multipleOf",
        "uniqueItems",
        "minProperties",
        "maxProperties",
        "contains",
        "minContains",
        "maxContains",
        "prefixItems",
        "unevaluatedItems",
        "patternProperties",
        "propertyNames",
        "dependentRequired",
        "dependentSchemas",
        "unevaluatedProperties",
        "const",
        "not",
        "contentEncoding",
        "contentMediaType",
    )
)
RECORDED_SECURITY_CONSTRAINTS = frozenset(
    (
        "minimum",
        "maximum",
        "minLength",
        "maxLength",
        "minItems",
        "maxItems",
        "format",
        "pattern",
    )
)
FIELDS = (
    "model",
    "path",
    "required",
    "types",
    "minimum",
    "maximum",
    "min_length",
    "max_length",
    "min_items",
    "max_items",
    "format",
    "pattern",
    "known_values",
)


def load_spec(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read {path}: {error}") from error
    if not isinstance(value, dict) or not isinstance(value.get("paths"), dict):
        raise ValueError("Cloud specification has no paths object")
    return value


def clean_schema(value: Any) -> Any:
    if isinstance(value, dict):
        return {
            key: clean_schema(item)
            for key, item in sorted(value.items())
            if key not in {"description", "example", "examples", "externalDocs"}
        }
    if isinstance(value, list):
        return [clean_schema(item) for item in value]
    return value


def success_schema(operation: dict[str, Any]) -> dict[str, Any]:
    responses = operation.get("responses")
    if not isinstance(responses, dict):
        return {}
    successes = [value for key, value in responses.items() if key.startswith("2")]
    if len(successes) != 1 or not isinstance(successes[0], dict):
        return {}
    content = successes[0].get("content", {})
    if not isinstance(content, dict):
        return {}
    media = content.get("application/json", {})
    if not isinstance(media, dict):
        return {}
    schema = media.get("schema", {})
    return schema if isinstance(schema, dict) else {}


def collect_models(document: dict[str, Any]) -> dict[str, dict[str, Any]]:
    candidates: dict[str, list[dict[str, Any]]] = {}
    for path_item in document["paths"].values():
        if not isinstance(path_item, dict):
            continue
        for method in METHODS:
            operation = path_item.get(method)
            if not isinstance(operation, dict) or operation.get("deprecated") is True:
                continue
            properties = success_schema(operation).get("properties", {})
            if not isinstance(properties, dict):
                continue
            for root, schema in properties.items():
                model = MODEL_ROOTS.get(root)
                if model is None or not isinstance(schema, dict):
                    continue
                if schema.get("type") == "array":
                    schema = schema.get("items", {})
                if not isinstance(schema, dict):
                    raise ValueError(f"{model} has an invalid resource schema")
                candidates.setdefault(model, []).append(clean_schema(schema))

    if set(candidates) != EXPECTED_MODELS:
        missing = sorted(EXPECTED_MODELS.difference(candidates))
        raise ValueError(f"missing Cloud resource schemas: {', '.join(missing)}")
    output = {}
    for model, schemas in candidates.items():
        canonical = schemas[0]
        if any(schema != canonical for schema in schemas[1:]):
            raise ValueError(f"{model} response schemas are not structurally identical")
        output[model] = canonical
    return output


def schema_types(schema: dict[str, Any]) -> list[str]:
    value = schema.get("type")
    if isinstance(value, str):
        return [value]
    if isinstance(value, list) and all(isinstance(item, str) for item in value):
        return sorted(value)
    if "properties" in schema or "allOf" in schema:
        return ["object"]
    if "oneOf" in schema:
        return ["object"]
    raise ValueError("schema node has no supported type")


def merge_all_of(schema: dict[str, Any]) -> dict[str, Any]:
    branches = schema.get("allOf")
    if not isinstance(branches, list):
        return schema
    merged: dict[str, Any] = {key: value for key, value in schema.items() if key != "allOf"}
    properties: dict[str, Any] = {}
    required: set[str] = set()
    for branch in branches:
        if not isinstance(branch, dict):
            raise ValueError("allOf branch is not an object")
        branch = merge_all_of(branch)
        branch_properties = branch.get("properties", {})
        if not isinstance(branch_properties, dict):
            raise ValueError("allOf properties are invalid")
        properties.update(branch_properties)
        branch_required = branch.get("required", [])
        if not isinstance(branch_required, list):
            raise ValueError("allOf required list is invalid")
        required.update(branch_required)
    merged["type"] = "object"
    merged["properties"] = properties
    merged["required"] = sorted(required)
    return merged


def cell(value: Any) -> str:
    if value is None:
        return "-"
    text = str(value)
    if any(character in text for character in "\t\r\n\0"):
        raise ValueError(f"unsafe schema cell: {text!r}")
    return text


def validate_constraints(model: str, path: str, schema: dict[str, Any]) -> None:
    unsupported = sorted(UNSUPPORTED_SECURITY_CONSTRAINTS.intersection(schema))
    if unsupported:
        joined = ", ".join(unsupported)
        raise ValueError(f"{model}:{path} has unenforced constraints: {joined}")

    format_value = schema.get("format")
    if format_value is not None and format_value not in SUPPORTED_FORMATS:
        raise ValueError(f"{model}:{path} has unsupported format: {format_value!r}")
    types = set(schema_types(schema))
    expected_type = {
        "date-time": "string",
        "decimal": "string",
        "double": "number",
        "int32": "integer",
        "int64": "integer",
    }.get(format_value)
    if expected_type is not None and expected_type not in types:
        raise ValueError(
            f"{model}:{path} format {format_value!r} requires {expected_type}"
        )
    pattern = schema.get("pattern")
    if pattern is not None and pattern not in SUPPORTED_PATTERNS:
        raise ValueError(f"{model}:{path} has unsupported pattern: {pattern!r}")
    if pattern is not None and "string" not in types:
        raise ValueError(f"{model}:{path} pattern requires string")

    branches = schema.get("allOf", [])
    if isinstance(branches, list):
        security_keys = UNSUPPORTED_SECURITY_CONSTRAINTS | RECORDED_SECURITY_CONSTRAINTS
        for branch in branches:
            if not isinstance(branch, dict):
                continue
            hidden = sorted(security_keys.intersection(branch))
            if hidden:
                joined = ", ".join(hidden)
                raise ValueError(
                    f"{model}:{path} has unflattened allOf constraints: {joined}"
                )
            validate_constraints(model, path, branch)


def descriptor(model: str, path: str, required: bool, schema: dict[str, Any]) -> dict[str, str]:
    validate_constraints(model, path, schema)
    known = schema.get("enum", [])
    if not isinstance(known, list) or not all(isinstance(item, str) for item in known):
        known = []
    return {
        "model": model,
        "path": path,
        "required": "yes" if required else "no",
        "types": "|".join(schema_types(schema)),
        "minimum": cell(schema.get("minimum")),
        "maximum": cell(schema.get("maximum")),
        "min_length": cell(schema.get("minLength")),
        "max_length": cell(schema.get("maxLength")),
        "min_items": cell(schema.get("minItems")),
        "max_items": cell(schema.get("maxItems")),
        "format": cell(schema.get("format")),
        "pattern": cell(schema.get("pattern")),
        "known_values": "|".join(known) or "-",
    }


def walk_object(model: str, prefix: str, schema: dict[str, Any], rows: list[dict[str, str]]) -> None:
    validate_constraints(model, prefix or "<root>", schema)
    schema = merge_all_of(schema)
    validate_constraints(model, prefix or "<root>", schema)
    properties = schema.get("properties", {})
    required = schema.get("required", [])
    if not isinstance(properties, dict) or not isinstance(required, list):
        raise ValueError(f"{model} has invalid object metadata at {prefix or '<root>'}")
    for name, child in sorted(properties.items()):
        if not isinstance(name, str) or not isinstance(child, dict) or "/" in name:
            raise ValueError(f"{model} has an invalid field at {prefix or '<root>'}")
        path = f"{prefix}/{name}" if prefix else name
        rows.append(descriptor(model, path, name in required, child))
        walk_children(model, path, child, rows)


def walk_union(model: str, path: str, schema: dict[str, Any], rows: list[dict[str, str]]) -> None:
    validate_constraints(model, path, schema)
    discriminator = schema.get("discriminator", {})
    selector = discriminator.get("propertyName") if isinstance(discriminator, dict) else None
    branches = schema.get("oneOf")
    if not isinstance(selector, str) or not isinstance(branches, list):
        raise ValueError(f"{model} has an unsupported union at {path}")
    selector_values: set[str] = set()
    merged_branches = []
    for branch in branches:
        if not isinstance(branch, dict):
            raise ValueError(f"{model} union branch is invalid at {path}")
        validate_constraints(model, path, branch)
        branch = merge_all_of(branch)
        properties = branch.get("properties", {})
        selector_schema = properties.get(selector, {}) if isinstance(properties, dict) else {}
        values = selector_schema.get("enum", []) if isinstance(selector_schema, dict) else []
        if not isinstance(values, list) or len(values) != 1 or not isinstance(values[0], str):
            raise ValueError(f"{model} union selector is ambiguous at {path}")
        selector_values.add(values[0])
        merged_branches.append((values[0], branch))
    rows.append(
        descriptor(
            model,
            f"{path}[]/{selector}",
            True,
            {"type": "string", "enum": sorted(selector_values)},
        )
    )
    for value, branch in merged_branches:
        walk_object(model, f"{path}[{selector}={value}]", branch, rows)


def walk_children(model: str, path: str, schema: dict[str, Any], rows: list[dict[str, str]]) -> None:
    types = schema_types(schema)
    if "object" in types and "null" not in types:
        walk_object(model, path, schema, rows)
    elif "object" in types and schema.get("properties"):
        walk_object(model, path, schema, rows)
    if "array" not in types:
        return
    items = schema.get("items", {})
    if not isinstance(items, dict):
        raise ValueError(f"{model} array items are invalid at {path}")
    if "oneOf" in items:
        walk_union(model, path, items, rows)
        return
    items = merge_all_of(items)
    item_types = schema_types(items)
    if "object" in item_types:
        walk_object(model, f"{path}[]", items, rows)
    else:
        rows.append(descriptor(model, f"{path}[]", True, items))


def render(document: dict[str, Any]) -> str:
    rows: list[dict[str, str]] = []
    for model, schema in sorted(collect_models(document).items()):
        walk_object(model, "", schema, rows)
    lines = ["\t".join(FIELDS)]
    lines.extend("\t".join(row[field] for field in FIELDS) for row in rows)
    return "\n".join(lines) + "\n"


def example_value(schema: dict[str, Any]) -> Any:
    schema = merge_all_of(schema)
    if "oneOf" in schema:
        branches = schema["oneOf"]
        if not isinstance(branches, list) or not branches:
            raise ValueError("union has no example branch")
        return example_value(branches[0])
    types = schema_types(schema)
    nonnull = next((value for value in types if value != "null"), "null")
    if nonnull == "null":
        return None
    if nonnull == "boolean":
        return False
    if nonnull == "integer":
        minimum = schema.get("minimum", 1)
        return max(int(minimum), 1)
    if nonnull == "number":
        minimum = schema.get("minimum", 1.0)
        return max(float(minimum), 1.0)
    if nonnull == "string":
        format_value = schema.get("format")
        if format_value == "date-time":
            return "2026-01-01T00:00:00Z"
        if format_value == "decimal":
            return "1.0"
        known = schema.get("enum", [])
        if isinstance(known, list) and known and isinstance(known[0], str):
            return known[0]
        minimum = schema.get("minLength", 1)
        length = max(int(minimum), 1)
        maximum = schema.get("maxLength")
        if maximum is not None:
            length = min(length, int(maximum))
        return "x" * length
    if nonnull == "array":
        items = schema.get("items", {})
        if not isinstance(items, dict):
            raise ValueError("array fixture has invalid items")
        branches = items.get("oneOf")
        if isinstance(branches, list):
            return [example_value(branch) for branch in branches]
        return [example_value(items)]
    if nonnull == "object":
        properties = schema.get("properties", {})
        if not isinstance(properties, dict):
            raise ValueError("object fixture has invalid properties")
        return {
            name: example_value(child)
            for name, child in sorted(properties.items())
            if isinstance(child, dict)
        }
    raise ValueError(f"unsupported fixture type: {nonnull}")


def render_fixtures(document: dict[str, Any]) -> str:
    fixtures = {
        model: example_value(schema)
        for model, schema in sorted(collect_models(document).items())
    }
    return json.dumps(fixtures, sort_keys=True, separators=(",", ":")) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("cloud_spec", type=Path)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--fixture-output", type=Path, default=DEFAULT_FIXTURES)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    document = load_spec(args.cloud_spec)
    generated = render(document)
    fixtures = render_fixtures(document)
    if args.check:
        if args.output.read_text(encoding="ascii") != generated:
            raise SystemExit("Hetzner Cloud model schema lock is stale")
        if args.fixture_output.read_text(encoding="ascii") != fixtures:
            raise SystemExit("Hetzner Cloud model fixtures are stale")
        print("Hetzner Cloud model schema lock is current")
        return 0
    args.output.write_text(generated, encoding="ascii")
    args.fixture_output.write_text(fixtures, encoding="ascii")
    print(f"wrote {len(generated.splitlines()) - 1} Cloud model schema rows")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
