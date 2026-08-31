#!/usr/bin/env python3
"""Validate non-header authentication encoded in crates.io operations."""

from __future__ import annotations

from typing import Any

from cratesio_source_error import SourceLockError


SYNTHETIC_AUTH = {
    "accept_crate_owner_invitation_with_token": "owner_invitation_path_token",
    "confirm_user_email": "email_confirmation_path_token",
    "exchange_trustpub_token": "oidc_assertion_body",
}


def synthetic_auth(
    operation_id: str,
    path: str,
    item: dict[str, Any],
    operation: dict[str, Any],
    observed: list[str],
) -> str | None:
    if operation_id not in SYNTHETIC_AUTH:
        return None
    if observed != ["anonymous"]:
        raise SourceLockError(f"{operation_id} synthetic auth conflicts with upstream")
    if operation_id == "exchange_trustpub_token":
        _validate_oidc_body(operation)
    else:
        name = "token" if operation_id.startswith("accept_") else "email_token"
        _validate_path_token(operation_id, name, path, item, operation)
    return SYNTHETIC_AUTH[operation_id]


def _validate_oidc_body(operation: dict[str, Any]) -> None:
    body = operation.get("requestBody")
    content = body.get("content") if isinstance(body, dict) else None
    media = content.get("application/json") if isinstance(content, dict) else None
    schema = media.get("schema") if isinstance(media, dict) else None
    properties = schema.get("properties") if isinstance(schema, dict) else None
    required = schema.get("required") if isinstance(schema, dict) else None
    jwt = properties.get("jwt") if isinstance(properties, dict) else None
    if (
        not isinstance(body, dict)
        or body.get("required") is not True
        or not isinstance(schema, dict)
        or schema.get("type") != "object"
        or not isinstance(required, list)
        or "jwt" not in required
        or not isinstance(jwt, dict)
        or jwt.get("type") != "string"
    ):
        raise SourceLockError("exchange_trustpub_token lacks its required OIDC body")


def _validate_path_token(
    operation_id: str,
    name: str,
    path: str,
    item: dict[str, Any],
    operation: dict[str, Any],
) -> None:
    item_parameters = item.get("parameters", [])
    operation_parameters = operation.get("parameters", [])
    if not isinstance(item_parameters, list) or not isinstance(
        operation_parameters, list
    ):
        raise SourceLockError(f"{operation_id} has invalid path parameters")
    valid = any(
        isinstance(parameter, dict)
        and parameter.get("in") == "path"
        and parameter.get("name") == name
        and parameter.get("required") is True
        and isinstance(parameter.get("schema"), dict)
        and parameter["schema"].get("type") == "string"
        for parameter in [*item_parameters, *operation_parameters]
    )
    if path.count(f"{{{name}}}") != 1 or not valid:
        raise SourceLockError(f"{operation_id} lacks its required path token")
