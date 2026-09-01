#!/usr/bin/env python3
"""Validate the admitted OpenAPI 3.1 JSON Schema dialect."""

from __future__ import annotations

from typing import Any

from cratesio_source_error import SourceLockError


OAS_31_DIALECT = "https://spec.openapis.org/oas/3.1/dialect/base"


def validate_schema_dialects(document: dict[str, Any]) -> None:
    """Require the OAS 3.1 dialect at the document and schema-resource levels."""
    if document.get("jsonSchemaDialect", OAS_31_DIALECT) != OAS_31_DIALECT:
        raise SourceLockError("OpenAPI schema dialect is not reviewed")
    stack: list[Any] = [document]
    while stack:
        value = stack.pop()
        if isinstance(value, dict):
            if "$schema" in value and value["$schema"] != OAS_31_DIALECT:
                raise SourceLockError("OpenAPI nested schema dialect is not reviewed")
            stack.extend(value.values())
        elif isinstance(value, list):
            stack.extend(value)
