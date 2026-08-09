"""Fail-closed schema-key and ``allOf`` policy for Cloud model generation."""

from __future__ import annotations

from typing import Any


SUPPORTED_FORMATS = frozenset(("date-time", "decimal", "double", "int32", "int64"))
SUPPORTED_PATTERNS = frozenset(
    (
        r"^[a-z0-9]+(-?[a-z0-9]*)*$",
        r"^[a-zA-Z0-9 ./_-]+$",
        r"[a-zA-Z0-9-_,:<>+#!\(\)\[\]\{\} ]*",
        r"^\S(.*\S)?$",
    )
)
ANNOTATION_KEYS = frozenset(
    (
        "default",
        "deprecated",
        "description",
        "example",
        "examples",
        "externalDocs",
        "readOnly",
        "title",
        "writeOnly",
        "x-enumDescriptions",
    )
)
HANDLED_SCHEMA_KEYS = frozenset(
    (
        "additionalProperties",
        "allOf",
        "enum",
        "format",
        "items",
        "maxItems",
        "maxLength",
        "maximum",
        "minItems",
        "minLength",
        "minimum",
        "pattern",
        "properties",
        "required",
        "type",
    )
)
UNION_SCHEMA_KEYS = frozenset(("discriminator", "oneOf"))
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


def _merge_overlapping_property(
    name: str, existing: dict[str, Any], incoming: dict[str, Any]
) -> dict[str, Any]:
    existing_semantic = {
        key: value for key, value in existing.items() if key not in ANNOTATION_KEYS
    }
    incoming_semantic = {
        key: value for key, value in incoming.items() if key not in ANNOTATION_KEYS
    }
    if existing_semantic == incoming_semantic:
        return {**existing, **incoming}

    allowed = {"type", "enum"}
    if (
        set(existing_semantic).difference(allowed)
        or set(incoming_semantic).difference(allowed)
        or existing.get("type") != "string"
        or incoming.get("type") != "string"
    ):
        raise ValueError(f"allOf property {name!r} has unsupported overlap")

    existing_values = existing.get("enum")
    incoming_values = incoming.get("enum")
    if not isinstance(existing_values, list) or not isinstance(incoming_values, list):
        raise ValueError(f"allOf property {name!r} has unsupported overlap")
    if not all(isinstance(value, str) for value in existing_values + incoming_values):
        raise ValueError(f"allOf property {name!r} has invalid enum refinement")
    intersection = [value for value in existing_values if value in incoming_values]
    if not intersection:
        raise ValueError(f"allOf property {name!r} has an empty enum intersection")
    return {**existing, **incoming, "enum": intersection}


def merge_all_of(schema: dict[str, Any]) -> dict[str, Any]:
    branches = schema.get("allOf")
    if not isinstance(branches, list):
        return schema
    siblings = sorted(set(schema).difference(ANNOTATION_KEYS).difference(("allOf",)))
    if siblings:
        joined = ", ".join(siblings)
        raise ValueError(f"allOf has unsupported sibling schema keys: {joined}")
    merged: dict[str, Any] = {
        key: value for key, value in schema.items() if key in ANNOTATION_KEYS
    }
    properties: dict[str, Any] = {}
    required: set[str] = set()
    for branch in branches:
        if not isinstance(branch, dict):
            raise ValueError("allOf branch is not an object")
        branch = merge_all_of(branch)
        branch_properties = branch.get("properties", {})
        if not isinstance(branch_properties, dict):
            raise ValueError("allOf properties are invalid")
        branch_keys = set(branch).difference(ANNOTATION_KEYS)
        if branch_keys.difference(("type", "properties", "required")):
            raise ValueError("allOf branch has unsupported schema composition")
        if branch.get("type") != "object":
            raise ValueError("allOf branch is not an explicit object")
        for name, value in branch_properties.items():
            if not isinstance(name, str) or not isinstance(value, dict):
                raise ValueError("allOf properties are invalid")
            existing = properties.get(name)
            properties[name] = (
                value
                if existing is None
                else _merge_overlapping_property(name, existing, value)
            )
        branch_required = branch.get("required", [])
        if not isinstance(branch_required, list):
            raise ValueError("allOf required list is invalid")
        required.update(branch_required)
    merged["type"] = "object"
    merged["properties"] = properties
    merged["required"] = sorted(required)
    return merged
