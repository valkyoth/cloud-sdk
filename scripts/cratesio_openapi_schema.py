#!/usr/bin/env python3
"""Validate controls only at typed OpenAPI Schema and Reference positions."""

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


class _OpenApiValidator:
    def __init__(self, document: dict[str, Any]) -> None:
        self.document = document

    def local_reference(self, reference: Any) -> None:
        if not isinstance(reference, str):
            raise SourceLockError("OpenAPI reference must be a string")
        if not reference.startswith("#/"):
            raise SourceLockError(f"OpenAPI has external reference {reference!r}")
        current: Any = self.document
        for raw in reference[2:].split("/"):
            key = raw.replace("~1", "/").replace("~0", "~")
            if not isinstance(current, dict) or key not in current:
                raise SourceLockError(
                    f"OpenAPI has unresolved reference {reference!r}"
                )
            current = current[key]

    def object_reference(self, value: Any, label: str) -> dict[str, Any]:
        if not isinstance(value, dict):
            raise SourceLockError(f"OpenAPI {label} is invalid")
        if "$ref" in value:
            self.local_reference(value["$ref"])
        return value

    def schema_tree(self, root: Any) -> None:
        stack = [root]
        while stack:
            schema = stack.pop()
            if isinstance(schema, bool):
                continue
            if not isinstance(schema, dict):
                raise SourceLockError("OpenAPI Schema Object is invalid")
            if "$ref" in schema:
                self.local_reference(schema["$ref"])
            if "$dynamicRef" in schema:
                reference = schema["$dynamicRef"]
                if not isinstance(reference, str):
                    raise SourceLockError("OpenAPI $dynamicRef must be a string")
                raise SourceLockError(
                    "OpenAPI dynamic schema references are not supported"
                )
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

    def value_map(
        self, values: Any, validator: Callable[[Any], None], label: str
    ) -> None:
        if values is None:
            return
        if not isinstance(values, dict):
            raise SourceLockError(f"OpenAPI {label} must be an object")
        for value in values.values():
            validator(value)

    def reference_value(self, value: Any) -> None:
        self.object_reference(value, "reference-capable object")

    def examples(self, examples: Any) -> None:
        self.value_map(examples, self.reference_value, "examples")

    def content(self, content: Any) -> None:
        def validate_media(media: Any) -> None:
            if not isinstance(media, dict):
                raise SourceLockError("OpenAPI media type is invalid")
            if "schema" in media:
                self.schema_tree(media["schema"])
            self.examples(media.get("examples"))
            encoding = media.get("encoding")
            if encoding is None:
                return
            if not isinstance(encoding, dict):
                raise SourceLockError("OpenAPI encoding map is invalid")
            for value in encoding.values():
                if not isinstance(value, dict):
                    raise SourceLockError("OpenAPI encoding is invalid")
                self.headers(value.get("headers"))

        self.value_map(content, validate_media, "content")

    def parameter(self, parameter: Any) -> None:
        value = self.object_reference(parameter, "parameter")
        if "schema" in value:
            self.schema_tree(value["schema"])
        self.content(value.get("content"))
        self.examples(value.get("examples"))

    def parameters(self, parameters: Any) -> None:
        if parameters is None:
            return
        if not isinstance(parameters, list):
            raise SourceLockError("OpenAPI parameters must be an array")
        for parameter in parameters:
            self.parameter(parameter)

    def headers(self, headers: Any) -> None:
        self.value_map(headers, self.parameter, "headers")

    def request_body(self, body: Any) -> None:
        value = self.object_reference(body, "request body")
        self.content(value.get("content"))

    def response(self, response: Any) -> None:
        value = self.object_reference(response, "response")
        self.content(value.get("content"))
        self.headers(value.get("headers"))
        self.value_map(value.get("links"), self.reference_value, "links")

    def callback(self, callback: Any) -> None:
        value = self.object_reference(callback, "callback")
        for expression, item in value.items():
            if expression != "$ref" and not expression.startswith("x-"):
                self.path_item(item)

    def operation(self, operation: Any) -> None:
        if not isinstance(operation, dict):
            raise SourceLockError("OpenAPI operation is invalid")
        self.parameters(operation.get("parameters"))
        if "requestBody" in operation:
            self.request_body(operation["requestBody"])
        self.value_map(operation.get("responses"), self.response, "responses")
        self.value_map(operation.get("callbacks"), self.callback, "callbacks")

    def path_item(self, item: Any) -> None:
        value = self.object_reference(item, "path item")
        self.parameters(value.get("parameters"))
        for method in HTTP_METHODS & value.keys():
            self.operation(value[method])

    def components(self, components: Any) -> None:
        if components is None:
            return
        if not isinstance(components, dict):
            raise SourceLockError("OpenAPI components must be an object")
        validators = {
            "callbacks": self.callback,
            "examples": self.reference_value,
            "headers": self.parameter,
            "links": self.reference_value,
            "parameters": self.parameter,
            "pathItems": self.path_item,
            "requestBodies": self.request_body,
            "responses": self.response,
            "schemas": self.schema_tree,
            "securitySchemes": self.reference_value,
        }
        for key, validator in validators.items():
            self.value_map(components.get(key), validator, f"component {key}")

    def document_tree(self) -> None:
        if self.document.get("jsonSchemaDialect", OAS_31_DIALECT) != OAS_31_DIALECT:
            raise SourceLockError("OpenAPI schema dialect is not reviewed")
        self.components(self.document.get("components"))
        self.value_map(self.document.get("paths"), self.path_item, "paths")
        self.value_map(self.document.get("webhooks"), self.path_item, "webhooks")


def validate_schema_tree(root: Any, document: dict[str, Any] | None = None) -> None:
    """Validate controls within one Schema Object tree."""
    _OpenApiValidator(document or {}).schema_tree(root)


def validate_schema_dialects(document: dict[str, Any]) -> None:
    """Validate dialects and local refs at typed OpenAPI control positions."""
    _OpenApiValidator(document).document_tree()
