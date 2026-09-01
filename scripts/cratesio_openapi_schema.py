#!/usr/bin/env python3
"""Validate JSON Schema dialects only at OpenAPI Schema Object positions."""

from __future__ import annotations

from typing import Any, Callable

from cratesio_source_error import SourceLockError


OAS_31_DIALECT = "https://spec.openapis.org/oas/3.1/dialect/base"
HTTP_METHODS = {"delete", "get", "head", "options", "patch", "post", "put", "trace"}
SINGLE_SUBSCHEMAS = {
    "additionalProperties",
    "contains",
    "contentSchema",
    "else",
    "if",
    "items",
    "not",
    "propertyNames",
    "then",
    "unevaluatedItems",
    "unevaluatedProperties",
}
ARRAY_SUBSCHEMAS = {"allOf", "anyOf", "oneOf", "prefixItems"}
MAP_SUBSCHEMAS = {"$defs", "dependentSchemas", "patternProperties", "properties"}


def validate_schema_tree(root: Any) -> None:
    """Validate dialects and container shapes within one Schema Object tree."""
    stack = [root]
    while stack:
        schema = stack.pop()
        if isinstance(schema, bool):
            continue
        if not isinstance(schema, dict):
            raise SourceLockError("OpenAPI Schema Object is invalid")
        if schema.get("$schema", OAS_31_DIALECT) != OAS_31_DIALECT:
            raise SourceLockError("OpenAPI nested schema dialect is not reviewed")
        for key in SINGLE_SUBSCHEMAS:
            if key in schema:
                stack.append(schema[key])
        for key in ARRAY_SUBSCHEMAS:
            if key not in schema:
                continue
            values = schema[key]
            if not isinstance(values, list):
                raise SourceLockError("OpenAPI schema array is invalid")
            stack.extend(values)
        for key in MAP_SUBSCHEMAS:
            if key not in schema:
                continue
            values = schema[key]
            if not isinstance(values, dict):
                raise SourceLockError("OpenAPI schema map is invalid")
            stack.extend(values.values())


def _validate_map(values: Any, validator: Callable[[Any], None], label: str) -> None:
    if values is None:
        return
    if not isinstance(values, dict):
        raise SourceLockError(f"OpenAPI {label} must be an object")
    for value in values.values():
        validator(value)


def _validate_content(content: Any) -> None:
    def validate_media(media: Any) -> None:
        if not isinstance(media, dict):
            raise SourceLockError("OpenAPI media type is invalid")
        if "schema" in media:
            validate_schema_tree(media["schema"])
        encoding = media.get("encoding")
        if encoding is None:
            return
        if not isinstance(encoding, dict):
            raise SourceLockError("OpenAPI encoding map is invalid")
        for value in encoding.values():
            if not isinstance(value, dict):
                raise SourceLockError("OpenAPI encoding is invalid")
            _validate_headers(value.get("headers"))

    _validate_map(content, validate_media, "content")


def _validate_parameter(parameter: Any) -> None:
    if not isinstance(parameter, dict):
        raise SourceLockError("OpenAPI parameter is invalid")
    if "schema" in parameter:
        validate_schema_tree(parameter["schema"])
    _validate_content(parameter.get("content"))


def _validate_parameters(parameters: Any) -> None:
    if parameters is None:
        return
    if not isinstance(parameters, list):
        raise SourceLockError("OpenAPI parameters must be an array")
    for parameter in parameters:
        _validate_parameter(parameter)


def _validate_headers(headers: Any) -> None:
    _validate_map(headers, _validate_parameter, "headers")


def _validate_request_body(body: Any) -> None:
    if not isinstance(body, dict):
        raise SourceLockError("OpenAPI request body is invalid")
    _validate_content(body.get("content"))


def _validate_response(response: Any) -> None:
    if not isinstance(response, dict):
        raise SourceLockError("OpenAPI response is invalid")
    _validate_content(response.get("content"))
    _validate_headers(response.get("headers"))


def _validate_callback(callback: Any) -> None:
    if not isinstance(callback, dict):
        raise SourceLockError("OpenAPI callback is invalid")
    for expression, item in callback.items():
        if expression != "$ref" and not expression.startswith("x-"):
            _validate_path_item(item)


def _validate_operation(operation: Any) -> None:
    if not isinstance(operation, dict):
        raise SourceLockError("OpenAPI operation is invalid")
    _validate_parameters(operation.get("parameters"))
    if "requestBody" in operation:
        _validate_request_body(operation["requestBody"])
    _validate_map(operation.get("responses"), _validate_response, "responses")
    _validate_map(operation.get("callbacks"), _validate_callback, "callbacks")


def _validate_path_item(item: Any) -> None:
    if not isinstance(item, dict):
        raise SourceLockError("OpenAPI path item is invalid")
    _validate_parameters(item.get("parameters"))
    for method in HTTP_METHODS & item.keys():
        _validate_operation(item[method])


def _validate_components(components: Any) -> None:
    if components is None:
        return
    if not isinstance(components, dict):
        raise SourceLockError("OpenAPI components must be an object")
    validators = {
        "callbacks": _validate_callback,
        "headers": _validate_parameter,
        "parameters": _validate_parameter,
        "pathItems": _validate_path_item,
        "requestBodies": _validate_request_body,
        "responses": _validate_response,
        "schemas": validate_schema_tree,
    }
    for key, validator in validators.items():
        _validate_map(components.get(key), validator, f"component {key}")


def validate_schema_dialects(document: dict[str, Any]) -> None:
    """Require the OAS dialect at the root and every actual Schema Object."""
    if document.get("jsonSchemaDialect", OAS_31_DIALECT) != OAS_31_DIALECT:
        raise SourceLockError("OpenAPI schema dialect is not reviewed")
    _validate_components(document.get("components"))
    _validate_map(document.get("paths"), _validate_path_item, "paths")
    _validate_map(document.get("webhooks"), _validate_path_item, "webhooks")
