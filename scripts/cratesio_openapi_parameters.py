#!/usr/bin/env python3
"""Validate crates.io OpenAPI path declarations and stable wire semantics."""

from __future__ import annotations

import re
from collections import Counter
from typing import Any

from cratesio_source_error import SourceLockError


PATH_PARAMETER = re.compile(r"\{([A-Za-z_][A-Za-z0-9_]*)\}")
STABLE_PATH_SCHEMAS = {
    ("add_owners", "name"): {"type": "string"},
    ("list_owners", "name"): {"type": "string"},
    ("remove_owners", "name"): {"type": "string"},
    ("unyank_version", "name"): {"type": "string"},
    ("unyank_version", "version"): {"type": "string"},
    ("yank_version", "name"): {"type": "string"},
    ("yank_version", "version"): {"type": "string"},
}


def _validate_stable_semantics(
    parameter: dict[str, Any], operation_id: str
) -> None:
    name = parameter.get("name")
    expected_schema = STABLE_PATH_SCHEMAS.get((operation_id, name))
    if expected_schema is None or parameter.get("schema") != expected_schema:
        raise SourceLockError(
            f"{operation_id}.{name} changed its stable parameter schema"
        )
    if (
        "content" in parameter
        or parameter.get("style", "simple") != "simple"
        or parameter.get("explode", False) is not False
        or parameter.get("allowReserved", False) is not False
    ):
        raise SourceLockError(
            f"{operation_id}.{name} has incompatible path serialization"
        )


def validate_path_parameters(
    path: str,
    item: dict[str, Any],
    operation: dict[str, Any],
    operation_id: str,
    stable: bool,
) -> None:
    """Require exact declarations and Cargo-compatible stable serialization."""
    expected = PATH_PARAMETER.findall(path)
    if path.count("{") != len(expected) or path.count("}") != len(expected):
        raise SourceLockError(f"{operation_id} has a malformed path template")
    item_parameters = item.get("parameters", [])
    operation_parameters = operation.get("parameters", [])
    if not isinstance(item_parameters, list) or not isinstance(
        operation_parameters, list
    ):
        raise SourceLockError(f"{operation_id} parameters must be arrays")
    declared = []
    for parameter in [*item_parameters, *operation_parameters]:
        if not isinstance(parameter, dict):
            raise SourceLockError(f"{operation_id} has an invalid parameter")
        if "$ref" in parameter:
            raise SourceLockError(f"{operation_id} parameter references are not supported")
        if parameter.get("in") != "path":
            continue
        if parameter.get("required") is not True:
            raise SourceLockError(f"{operation_id} has a non-required path parameter")
        name = parameter.get("name")
        if not isinstance(name, str):
            raise SourceLockError(f"{operation_id} has an unnamed path parameter")
        if stable:
            _validate_stable_semantics(parameter, operation_id)
        declared.append(name)
    expected_counts = Counter(expected)
    declared_counts = Counter(declared)
    if (
        any(count != 1 for count in expected_counts.values())
        or any(count != 1 for count in declared_counts.values())
        or expected_counts != declared_counts
    ):
        raise SourceLockError(
            f"{operation_id} path template and declarations differ"
        )
