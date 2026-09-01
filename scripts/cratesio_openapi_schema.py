#!/usr/bin/env python3
"""Validate controls only at typed OpenAPI Schema and Reference positions."""

from __future__ import annotations

from typing import Any, Callable, NoReturn
from urllib.parse import unquote_to_bytes

from cratesio_source_error import SourceLockError


OAS_31_DIALECT = "https://spec.openapis.org/oas/3.1/dialect/base"
MAX_REFERENCE_DEPTH = 128
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
        self.visited: dict[str, set[int]] = {}
        self.reference_depth = 0

    @staticmethod
    def _unresolved(reference: str) -> NoReturn:
        raise SourceLockError(f"OpenAPI has unresolved reference {reference!r}")

    @staticmethod
    def _pointer_token(raw: str, reference: str) -> str:
        output: list[str] = []
        index = 0
        while index < len(raw):
            if raw[index] != "~":
                output.append(raw[index])
                index += 1
                continue
            if index + 1 >= len(raw) or raw[index + 1] not in "01":
                _OpenApiValidator._unresolved(reference)
            output.append("~" if raw[index + 1] == "0" else "/")
            index += 2
        return "".join(output)

    @staticmethod
    def _fragment(reference: str) -> str:
        encoded = reference[2:]
        index = 0
        while index < len(encoded):
            if encoded[index] != "%":
                index += 1
                continue
            if (
                index + 2 >= len(encoded)
                or encoded[index + 1] not in "0123456789abcdefABCDEF"
                or encoded[index + 2] not in "0123456789abcdefABCDEF"
            ):
                _OpenApiValidator._unresolved(reference)
            index += 3
        try:
            return unquote_to_bytes(encoded).decode("utf-8", errors="strict")
        except UnicodeDecodeError as error:
            raise SourceLockError(
                f"OpenAPI has unresolved reference {reference!r}"
            ) from error

    def resolve_local_reference(self, reference: Any) -> Any:
        if not isinstance(reference, str):
            raise SourceLockError("OpenAPI reference must be a string")
        if not reference.startswith("#/"):
            raise SourceLockError(f"OpenAPI has external reference {reference!r}")
        current: Any = self.document
        for raw in self._fragment(reference).split("/"):
            token = self._pointer_token(raw, reference)
            if isinstance(current, dict):
                if token not in current:
                    self._unresolved(reference)
                current = current[token]
                continue
            if isinstance(current, list):
                if (
                    not current
                    or not token.isascii()
                    or not token.isdigit()
                    or (len(token) > 1 and token.startswith("0"))
                    or len(token) > len(str(len(current) - 1))
                ):
                    self._unresolved(reference)
                position = int(token)
                if position >= len(current):
                    self._unresolved(reference)
                current = current[position]
                continue
            self._unresolved(reference)
        return current

    def object_reference(
        self,
        value: Any,
        label: str,
        context: str,
        validator: Callable[[Any], None],
    ) -> dict[str, Any] | None:
        if not isinstance(value, dict):
            raise SourceLockError(f"OpenAPI {label} is invalid")
        identity = id(value)
        visited = self.visited.setdefault(context, set())
        if identity in visited:
            return None
        visited.add(identity)
        if "$ref" in value:
            if self.reference_depth >= MAX_REFERENCE_DEPTH:
                raise SourceLockError(
                    "OpenAPI reference depth exceeds reviewed limit"
                )
            target = self.resolve_local_reference(value["$ref"])
            self.reference_depth += 1
            try:
                validator(target)
            finally:
                self.reference_depth -= 1
        return value

    def schema_tree(self, root: Any) -> None:
        stack = [root]
        while stack:
            schema = stack.pop()
            if isinstance(schema, bool):
                continue
            if not isinstance(schema, dict):
                raise SourceLockError("OpenAPI Schema Object is invalid")
            identity = id(schema)
            visited = self.visited.setdefault("schema", set())
            if identity in visited:
                continue
            visited.add(identity)
            if "$ref" in schema:
                target = self.resolve_local_reference(schema["$ref"])
                if not isinstance(target, (dict, bool)):
                    raise SourceLockError("OpenAPI schema reference target is invalid")
                stack.append(target)
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

    def example(self, value: Any) -> None:
        self.object_reference(value, "example", "example", self.example)

    def link(self, value: Any) -> None:
        self.object_reference(value, "link", "link", self.link)

    def security_scheme(self, value: Any) -> None:
        self.object_reference(
            value, "security scheme", "security scheme", self.security_scheme
        )

    def examples(self, examples: Any) -> None:
        self.value_map(examples, self.example, "examples")

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
        value = self.object_reference(
            parameter, "parameter", "parameter", self.parameter
        )
        if value is None:
            return
        if "schema" in value:
            self.schema_tree(value["schema"])
        self.content(value.get("content"))
        self.examples(value.get("examples"))

    def header(self, header: Any) -> None:
        value = self.object_reference(header, "header", "header", self.header)
        if value is None:
            return
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
        self.value_map(headers, self.header, "headers")

    def request_body(self, body: Any) -> None:
        value = self.object_reference(
            body, "request body", "request body", self.request_body
        )
        if value is None:
            return
        self.content(value.get("content"))

    def response(self, response: Any) -> None:
        value = self.object_reference(response, "response", "response", self.response)
        if value is None:
            return
        self.content(value.get("content"))
        self.headers(value.get("headers"))
        self.value_map(value.get("links"), self.link, "links")

    def callback(self, callback: Any) -> None:
        value = self.object_reference(callback, "callback", "callback", self.callback)
        if value is None:
            return
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
        value = self.object_reference(item, "path item", "path item", self.path_item)
        if value is None:
            return
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
            "examples": self.example,
            "headers": self.header,
            "links": self.link,
            "parameters": self.parameter,
            "pathItems": self.path_item,
            "requestBodies": self.request_body,
            "responses": self.response,
            "schemas": self.schema_tree,
            "securitySchemes": self.security_scheme,
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
