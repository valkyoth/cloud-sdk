"""Canonical Hetzner OpenAPI operation, parameter, and schema contracts."""

from __future__ import annotations

import hashlib
import json
from typing import Any


DOC_ONLY_KEYS = {"description", "summary", "externalDocs", "example", "examples"}
HTTP_METHODS = {"get", "post", "put", "patch", "delete"}


def clean_json(value: Any) -> Any:
    if isinstance(value, dict):
        return {
            key: clean_json(item)
            for key, item in sorted(value.items())
            if key not in DOC_ONLY_KEYS
        }
    if isinstance(value, list):
        return [clean_json(item) for item in value]
    return value


def digest(value: Any) -> str:
    payload = json.dumps(clean_json(value), sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()


def action_kind(method: str, path: str) -> str:
    if path.endswith("/actions"):
        return "action-list"
    if "/actions/{action_id}" in path:
        return "resource-action-get"
    if "/actions/{id}" in path:
        return "action-get"
    if "/actions/" in path and method == "post":
        return "starts-action"
    if "/actions/" in path:
        return "action"
    return "none"


def parameter_array(value: Any, context: str) -> list[dict[str, Any]]:
    if value is None:
        return []
    if not isinstance(value, list):
        raise ValueError(f"{context} parameters must be an array")
    result = []
    for parameter in value:
        if not isinstance(parameter, dict):
            raise ValueError(f"{context} parameter must be an object")
        if "$ref" in parameter:
            raise ValueError(f"{context} parameter references are not supported")
        result.append(parameter)
    return result


def query_names(operation: dict[str, Any]) -> set[str]:
    names = set()
    for parameter in parameter_array(operation.get("parameters"), "operation"):
        if parameter.get("in") == "query":
            name = parameter.get("name")
            if not isinstance(name, str):
                raise ValueError("query parameter name must be text")
            names.add(name)
    return names


def operation_rows(api: str, document: dict[str, Any]) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    paths = document.get("paths", {})
    if not isinstance(paths, dict):
        raise SystemExit(f"{api} spec paths must be an object")
    for path, path_item in paths.items():
        if not isinstance(path, str) or not isinstance(path_item, dict):
            raise SystemExit(f"{api} spec contains an invalid path item")
        for method, operation in path_item.items():
            if method not in HTTP_METHODS:
                continue
            if not isinstance(operation, dict):
                raise SystemExit(f"{api} spec contains an invalid operation")
            try:
                queries = query_names(operation)
            except ValueError as error:
                raise SystemExit(
                    f"{api} spec contains invalid operation parameters"
                ) from error
            tags = operation.get("tags") or ["untagged"]
            operation_id = operation.get("operationId", "")
            if (
                not isinstance(tags, list)
                or not tags
                or not isinstance(tags[0], str)
                or not isinstance(operation_id, str)
            ):
                raise SystemExit(f"{api} spec contains invalid operation metadata")
            fingerprint_input = dict(operation)
            fingerprint_input.pop("deprecated", None)
            rows.append(
                {
                    "api": api,
                    "method": method.upper(),
                    "path": path,
                    "tag": tags[0],
                    "operation_id": operation_id,
                    "deprecated": "yes" if operation.get("deprecated") else "no",
                    "pagination": "yes"
                    if {"page", "per_page"}.issubset(queries)
                    else "no",
                    "sorting": "yes" if "sort" in queries else "no",
                    "action": action_kind(method, path),
                    "fingerprint": digest(fingerprint_input),
                }
            )
    return sorted(rows, key=lambda row: (row["api"], row["path"], row["method"]))


def json_cell(value: Any) -> str:
    return json.dumps(value, ensure_ascii=True, sort_keys=True, separators=(",", ":"))


def type_cell(schema: dict[str, Any], key: str = "type") -> str:
    value = schema.get(key, "")
    if isinstance(value, str):
        return value
    if isinstance(value, list) and all(isinstance(item, str) for item in value):
        return json_cell(value)
    if value == "":
        return ""
    raise ValueError("parameter schema type must be text or an array of text")


def effective_parameters(
    path_item: dict[str, Any], operation: dict[str, Any]
) -> list[dict[str, Any]]:
    indexed: dict[tuple[str, str], dict[str, Any]] = {}
    for context, parameters in (
        ("path", path_item.get("parameters")),
        ("operation", operation.get("parameters")),
    ):
        for parameter in parameter_array(parameters, context):
            location = parameter.get("in")
            name = parameter.get("name")
            if not isinstance(location, str) or not isinstance(name, str):
                raise ValueError(f"{context} parameter identity must be text")
            indexed[(location, name)] = parameter
    return [indexed[key] for key in sorted(indexed)]


def parameter_rows(api: str, document: dict[str, Any]) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    paths = document.get("paths", {})
    if not isinstance(paths, dict):
        raise SystemExit(f"{api} spec paths must be an object")
    for path, path_item in paths.items():
        if not isinstance(path, str) or not isinstance(path_item, dict):
            raise SystemExit(f"{api} spec contains an invalid path item")
        for method, operation in path_item.items():
            if method not in HTTP_METHODS:
                continue
            if not isinstance(operation, dict):
                raise SystemExit(f"{api} spec contains an invalid operation")
            operation_id = operation.get("operationId", "")
            if not isinstance(operation_id, str):
                raise SystemExit(f"{api} spec contains invalid operation metadata")
            try:
                parameters = effective_parameters(path_item, operation)
                rows.extend(
                    parameter_row(api, method, path, operation_id, parameter)
                    for parameter in parameters
                )
            except ValueError as error:
                raise SystemExit(f"{api} spec contains invalid parameters") from error
    return sorted(
        rows,
        key=lambda row: (
            row["api"],
            row["path"],
            row["method"],
            row["in"],
            row["name"],
        ),
    )


def parameter_row(
    api: str,
    method: str,
    path: str,
    operation_id: str,
    parameter: dict[str, Any],
) -> dict[str, str]:
    location = parameter["in"]
    name = parameter["name"]
    schema = parameter.get("schema", {})
    if not isinstance(schema, dict):
        raise ValueError("parameter schema must be an object")
    items = schema.get("items", {})
    if not isinstance(items, dict):
        raise ValueError("parameter items must be an object")
    style = parameter.get("style")
    if style is None:
        style = "form" if location in {"query", "cookie"} else "simple"
    if not isinstance(style, str):
        raise ValueError("parameter style must be text")
    explode = parameter.get("explode")
    if explode is None:
        explode = style == "form"
    if not isinstance(explode, bool):
        raise ValueError("parameter explode must be boolean")
    constraints = {
        key: schema[key]
        for key in (
            "minimum",
            "maximum",
            "exclusiveMinimum",
            "exclusiveMaximum",
            "minLength",
            "maxLength",
            "minItems",
            "maxItems",
            "uniqueItems",
            "pattern",
        )
        if key in schema
    }
    enum = items.get("enum", schema.get("enum", []))
    return {
        "api": api,
        "method": method.upper(),
        "path": path,
        "operation_id": operation_id,
        "in": location,
        "name": name,
        "required": "yes" if parameter.get("required") else "no",
        "schema_type": type_cell(schema),
        "schema_format": str(schema.get("format", "")),
        "items_type": type_cell(items),
        "style": style,
        "explode": "yes" if explode else "no",
        "enum": json_cell(enum),
        "constraints": json_cell(constraints),
        "fingerprint": digest(parameter),
    }


def schema_rows(api: str, document: dict[str, Any]) -> list[dict[str, str]]:
    components = document.get("components", {})
    if not isinstance(components, dict):
        raise SystemExit(f"{api} spec components must be an object")
    schemas = components.get("schemas", {})
    if not isinstance(schemas, dict) or any(
        not isinstance(name, str) for name in schemas
    ):
        raise SystemExit(f"{api} spec schemas must be a text-keyed object")
    rows = [
        {"api": api, "schema": name, "fingerprint": digest(schema)}
        for name, schema in schemas.items()
    ]
    return sorted(rows, key=lambda row: (row["api"], row["schema"]))
