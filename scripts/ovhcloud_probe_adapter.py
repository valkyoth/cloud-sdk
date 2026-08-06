#!/usr/bin/env python3
"""Normalize the reviewed OVHcloud API v2 probe sources."""

from __future__ import annotations

import hashlib
import json
from typing import Any


class OvhcloudProbeError(RuntimeError):
    """Official probe sources do not match the reviewed shape."""


CANDIDATE_PATHS = (
    "/iam/permissionsGroup",
    "/iam/permissionsGroup/{permissionsGroupURN}",
    "/iam/policy",
    "/iam/policy/{policyId}",
    "/iam/resource",
    "/iam/resource/{resourceURN}",
    "/iam/resourceGroup",
    "/iam/resourceGroup/{groupId}",
)


def _pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise OvhcloudProbeError("OVHcloud JSON contains a duplicate key")
        result[key] = value
    return result


def _json(payload: bytes, label: str) -> dict[str, Any]:
    try:
        value = json.loads(payload, object_pairs_hook=_pairs)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise OvhcloudProbeError(f"{label} is not strict UTF-8 JSON") from error
    if not isinstance(value, dict):
        raise OvhcloudProbeError(f"{label} root is not an object")
    return value


def _text(payload: bytes, label: str) -> str:
    try:
        return payload.decode("utf-8")
    except UnicodeDecodeError as error:
        raise OvhcloudProbeError(f"{label} is not UTF-8") from error


def _require(text: str, markers: tuple[str, ...], label: str) -> None:
    if any(marker not in text for marker in markers):
        raise OvhcloudProbeError(f"{label} is missing a reviewed contract")


def _digest(value: Any) -> str:
    encoded = json.dumps(
        value, ensure_ascii=True, sort_keys=True, separators=(",", ":")
    ).encode("ascii")
    return hashlib.sha256(encoded).hexdigest()


def _source_digest(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def _candidate_id(path: str) -> str:
    return (
        path.removeprefix("/")
        .lower()
        .replace("/{", "/by-")
        .replace("}", "")
    )


def _api_sections(index: dict[str, Any]) -> list[str]:
    if index.get("basePath") != "https://api.eu.ovhcloud.com/v2":
        raise OvhcloudProbeError("OVHcloud API index authority is invalid")
    apis = index.get("apis")
    if not isinstance(apis, list):
        raise OvhcloudProbeError("OVHcloud API index is invalid")
    sections: list[str] = []
    for entry in apis:
        if (
            not isinstance(entry, dict)
            or not isinstance(entry.get("path"), str)
            or entry.get("schema") != "{path}.{format}"
            or entry.get("format") != ["json", "yaml"]
        ):
            raise OvhcloudProbeError("OVHcloud API index entry is invalid")
        sections.append(entry["path"])
    if len(sections) != len(set(sections)) or "/iam" not in sections:
        raise OvhcloudProbeError("OVHcloud API index is incomplete")
    return sorted(sections)


def _candidate_operations(schema: dict[str, Any]) -> list[dict[str, Any]]:
    if (
        schema.get("basePath") != "https://api.eu.ovhcloud.com/v2"
        or schema.get("apiVersion") != "1.0"
    ):
        raise OvhcloudProbeError("OVHcloud IAM schema identity is invalid")
    apis = schema.get("apis")
    if not isinstance(apis, list):
        raise OvhcloudProbeError("OVHcloud IAM operations are invalid")
    by_path: dict[str, list[dict[str, Any]]] = {}
    for item in apis:
        if not isinstance(item, dict) or not isinstance(item.get("path"), str):
            raise OvhcloudProbeError("OVHcloud IAM path is invalid")
        operations = item.get("operations")
        if not isinstance(operations, list) or not all(
            isinstance(operation, dict) for operation in operations
        ):
            raise OvhcloudProbeError("OVHcloud IAM operation is invalid")
        by_path[item["path"]] = operations

    selected: list[dict[str, Any]] = []
    for path in CANDIDATE_PATHS:
        matches = [
            operation
            for operation in by_path.get(path, [])
            if operation.get("httpMethod") == "GET"
        ]
        if len(matches) != 1:
            raise OvhcloudProbeError("OVHcloud probe candidate is unavailable")
        operation = matches[0]
        status = operation.get("apiStatus")
        parameters = operation.get("parameters")
        if (
            not isinstance(status, dict)
            or status.get("value") != "PRODUCTION"
            or operation.get("noAuthentication") is not False
            or not isinstance(operation.get("responseType"), str)
            or not isinstance(parameters, list)
        ):
            raise OvhcloudProbeError("OVHcloud probe candidate is not stable read-only")
        headers = []
        for parameter in parameters:
            if not isinstance(parameter, dict):
                raise OvhcloudProbeError("OVHcloud candidate parameter is invalid")
            if parameter.get("paramType") == "header":
                name = parameter.get("name")
                if not isinstance(name, str):
                    raise OvhcloudProbeError("OVHcloud candidate header is invalid")
                headers.append(name)
        actions = operation.get("iamActions", [])
        if not isinstance(actions, list):
            raise OvhcloudProbeError("OVHcloud candidate actions are invalid")
        action_names = []
        for action in actions:
            if not isinstance(action, dict) or not isinstance(action.get("name"), str):
                raise OvhcloudProbeError("OVHcloud candidate action is invalid")
            action_names.append(action["name"])
        values = {
            "actions": sorted(action_names),
            "authenticated": True,
            "headers": sorted(headers),
            "method": "GET",
            "path": path,
            "response_type": operation["responseType"],
            "stability": "production",
        }
        selected.append({"id": _candidate_id(path), "values": values})
    return selected


def _model_evidence(schema: dict[str, Any]) -> dict[str, Any]:
    models = schema.get("models")
    if not isinstance(models, dict) or not models:
        raise OvhcloudProbeError("OVHcloud IAM models are invalid")
    rows = []
    for name, model in models.items():
        if not isinstance(name, str) or not isinstance(model, dict):
            raise OvhcloudProbeError("OVHcloud IAM model is invalid")
        rows.append({"model": model, "name": name})
    rows.sort(key=lambda row: row["name"])
    return {"count": len(rows), "sha256": _digest(rows)}


def build_observation(lock: dict[str, Any], payloads: dict[str, bytes]) -> dict[str, Any]:
    """Build the complete probe observation from authenticated sources."""
    expected = {
        "api-index",
        "api-v2-principles",
        "iam-schema",
        "oauth2-service-account",
    }
    if set(payloads) != expected:
        raise OvhcloudProbeError("OVHcloud probe source set is incomplete")
    index = _json(payloads["api-index"], "OVHcloud API index")
    schema = _json(payloads["iam-schema"], "OVHcloud IAM schema")
    principles = _text(payloads["api-v2-principles"], "OVHcloud API principles")
    oauth = _text(payloads["oauth2-service-account"], "OVHcloud OAuth2 guide")
    _require(
        principles,
        (
            "X-Schemas-Version",
            "https://eu.api.ovh.com/v2/iam/policy",
            "X-Pagination-Size",
            "X-Pagination-Cursor-Next",
            "X-Pagination-Cursor",
            "a route `/task`",
            "path `/event`",
        ),
        "OVHcloud API principles",
    )
    _require(
        oauth,
        (
            "OAuth2 *client credentials* flow",
            "https://www.ovh.com/auth/oauth2/token",
            "https://ca.ovh.com/auth/oauth2/token",
            '"access_token"',
            '"token_type":"Bearer"',
            '"expires_in":3599',
            "https://{eu|ca}.api.ovh.com/v1/",
        ),
        "OVHcloud OAuth2 guide",
    )
    sections = _api_sections(index)
    candidates = _candidate_operations(schema)
    candidate_digest = _digest(candidates)
    paginated = [
        row["id"]
        for row in candidates
        if row["values"]["headers"]
        == ["X-Pagination-Cursor", "X-Pagination-Size"]
    ]
    principles_sha = _source_digest(payloads["api-v2-principles"])
    oauth_sha = _source_digest(payloads["oauth2-service-account"])
    contracts = {
        "authentication": [
            {
                "id": "oauth2-client-credentials",
                "values": {
                    "flow": "client_credentials",
                    "request_media": "application/x-www-form-urlencoded",
                    "response_fields": [
                        "access_token",
                        "expires_in",
                        "scope",
                        "token_type",
                    ],
                    "scheme": "bearer",
                    "source_sha256": oauth_sha,
                    "token_endpoints": [
                        {
                            "region": "ca",
                            "url": "https://ca.ovh.com/auth/oauth2/token",
                        },
                        {
                            "region": "eu",
                            "url": "https://www.ovh.com/auth/oauth2/token",
                        },
                    ],
                },
            }
        ],
        "cost": [
            {
                "id": "probe-read-only",
                "values": {
                    "candidate_count": len(candidates),
                    "candidates_sha256": candidate_digest,
                    "cost_approval": "not_applicable",
                    "methods": ["GET"],
                },
            }
        ],
        "endpoints": [
            {
                "id": "ca-api-v2",
                "values": {
                    "base_path": "/v2",
                    "host": "ca.api.ovh.com",
                    "region": "ca",
                    "token_host": "ca.ovh.com",
                },
            },
            {
                "id": "eu-api-v2",
                "values": {
                    "base_path": "/v2",
                    "host": "eu.api.ovh.com",
                    "region": "eu",
                    "token_host": "www.ovh.com",
                },
            },
            {
                "id": "eu-console-schema",
                "values": {
                    "base_path": "/v2",
                    "host": "api.eu.ovhcloud.com",
                    "sections_sha256": _digest(sections),
                },
            },
        ],
        "headers": [
            {
                "id": "cursor-pagination",
                "values": {
                    "request": ["X-Pagination-Cursor", "X-Pagination-Size"],
                    "response": ["X-Pagination-Cursor-Next"],
                    "source_sha256": principles_sha,
                },
            },
            {
                "id": "schema-version-validation",
                "values": {
                    "account_default_when_absent": True,
                    "request": "X-Schemas-Version",
                    "source_sha256": principles_sha,
                    "use": "validation_only",
                },
            },
        ],
        "idempotency": [
            {
                "id": "probe-read-only",
                "values": {
                    "candidates_sha256": candidate_digest,
                    "key": "not_applicable",
                    "methods": ["GET"],
                },
            }
        ],
        "operations": candidates,
        "pagination": [
            {
                "id": "iam-cursor",
                "values": {
                    "cursor_request": "X-Pagination-Cursor",
                    "next_response": "X-Pagination-Cursor-Next",
                    "operation_count": len(paginated),
                    "operations_sha256": _digest(sorted(paginated)),
                    "size_request": "X-Pagination-Size",
                    "terminal": "next_header_absent",
                },
            }
        ],
        "retry": [
            {
                "id": "probe-read-only",
                "values": {
                    "automatic_policy": "caller_supplied",
                    "candidates_sha256": candidate_digest,
                    "methods": ["GET"],
                },
            }
        ],
        "schemas": [
            {
                "id": "api-sections",
                "values": {"count": len(sections), "sha256": _digest(sections)},
            },
            {"id": "iam-models", "values": _model_evidence(schema)},
            {
                "id": "task-event-contract",
                "values": {
                    "event_path": "/event",
                    "source_sha256": principles_sha,
                    "task_path": "/task",
                },
            },
        ],
    }
    observation = {
        "contracts": contracts,
        "format": "cloud-sdk-provider-observation/v1",
        "plugin": dict(lock["plugin"]),
        "provider": lock["provider"],
        "sources": [dict(source) for source in lock["sources"]],
    }
    for source in observation["sources"]:
        source["sha256"] = _source_digest(payloads[source["id"]])
    return observation
