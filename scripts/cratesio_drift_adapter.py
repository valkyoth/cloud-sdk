#!/usr/bin/env python3
"""Normalize crates.io sources into provider-generic drift contracts."""

from __future__ import annotations

import hashlib
import re
from typing import Any

from cratesio_source_lock import (
    ADMITTED_METHODS,
    CARGO_CONTRACTS,
    cargo_rows,
    digest,
    operation_rows,
    parse_json,
    validate_source_evidence,
)
from cratesio_reviewed_policy import policy_observation


class CratesioAdapterError(ValueError):
    """crates.io evidence cannot form a complete semantic observation."""


SOURCE_IDS = {
    "cargo",
    "openapi",
    "openapi-source",
    "policy",
    "policy-current",
    "policy-source",
}
PATH_PARAMETER = re.compile(r"\{[^{}]+\}")


def _row(identity: str, values: dict[str, Any]) -> dict[str, Any]:
    return {"id": identity, "values": values}


def _operation_nodes(document: dict[str, Any]) -> dict[str, tuple[str, str, dict[str, Any], dict[str, Any]]]:
    nodes: dict[str, tuple[str, str, dict[str, Any], dict[str, Any]]] = {}
    for path, item in document["paths"].items():
        for method in sorted(ADMITTED_METHODS & item.keys()):
            operation = item[method]
            operation_id = operation["operationId"]
            nodes[operation_id] = (method.upper(), path, item, operation)
    return nodes


def _operation_contracts(
    document: dict[str, Any], rows: list[dict[str, str]]
) -> tuple[list[dict[str, Any]], list[dict[str, Any]], list[dict[str, Any]]]:
    nodes = _operation_nodes(document)
    operations: list[dict[str, Any]] = []
    authentication: list[dict[str, Any]] = []
    schemas: list[dict[str, Any]] = []
    for row in rows:
        operation_id = row["operation_id"]
        method, path, item, operation = nodes[operation_id]
        operation_values = {
            "cargo_contract": row["cargo_contract"] == "yes",
            "classification": row["classification"],
            "deprecated": row["stability"].startswith("deprecated-"),
            "method": method,
            "path": path,
            "policy": row["policy"],
            "stability": row["stability"],
        }
        operation_values["fingerprint"] = digest(operation_values)
        operations.append(_row(f"openapi/{operation_id}", operation_values))
        authentication.append(
            _row(
                f"operation/{operation_id}",
                {
                    "admitted": row["admitted_auth"].split("|"),
                    "fingerprint": digest(
                        {
                            "admitted": row["admitted_auth"],
                            "observed": row["observed_auth"],
                        }
                    ),
                    "observed": row["observed_auth"].split("|"),
                },
            )
        )
        request = {
            "parameters": operation.get("parameters", []),
            "path_parameters": item.get("parameters", []),
            "request_body": operation.get("requestBody"),
        }
        schemas.append(
            _row(
                f"request/{operation_id}",
                {
                    "content_types": []
                    if row["request_media"] == "-"
                    else row["request_media"].split(","),
                    "fingerprint": digest(request),
                },
            )
        )
        schemas.append(
            _row(
                f"response/{operation_id}",
                {
                    "content_types": []
                    if row["response_media"] == "-"
                    else row["response_media"].split(","),
                    "fingerprint": row["response_sha256"],
                    "statuses": row["response_statuses"].split(","),
                },
            )
        )
    return operations, authentication, schemas


def _component_schemas(document: dict[str, Any]) -> list[dict[str, Any]]:
    components = document.get("components")
    schemas = components.get("schemas") if isinstance(components, dict) else None
    if not isinstance(schemas, dict):
        raise CratesioAdapterError("OpenAPI component schemas are missing")
    rows = []
    for name, schema in sorted(schemas.items()):
        if not isinstance(name, str) or not isinstance(schema, dict):
            raise CratesioAdapterError("OpenAPI component schema is invalid")
        identity = hashlib.sha256(name.encode("utf-8")).hexdigest()[:32]
        rows.append(
            _row(
                f"component/{identity}",
                {"fingerprint": digest(schema), "name": name},
            )
        )
    return rows


def _cargo_contracts(
    rows: list[dict[str, str]], operations: list[dict[str, str]]
) -> list[dict[str, Any]]:
    openapi = {row["operation_id"]: row for row in operations}
    contracts = []
    for row in rows:
        operation = openapi.get(row["openapi_operation_id"])
        matches = (
            operation is not None
            and operation["method"] == row["method"]
            and PATH_PARAMETER.sub("{}", operation["path"])
            == PATH_PARAMETER.sub("{}", row["path"])
            and operation["stability"] == "stable-cargo"
        )
        values = {
            "classification": row["classification"],
            "contract_fingerprint": row["contract_sha256"],
            "method": row["method"],
            "openapi_match": matches,
            "openapi_operation_id": row["openapi_operation_id"],
            "openapi_stability": "missing" if operation is None else operation["stability"],
            "path": row["path"],
            "policy": row["policy"],
        }
        values["fingerprint"] = digest(values)
        contracts.append(_row(f"cargo/{row['contract']}", values))
    return contracts


def _auth_scheme_contract(document: dict[str, Any]) -> dict[str, Any]:
    schemes = document["components"]["securitySchemes"]
    return _row(
        "schemes",
        {
            "fingerprint": digest(schemes),
            "names": sorted(schemes),
        },
    )


def _policy_contracts(policy: dict[str, Any]) -> dict[str, list[dict[str, Any]]]:
    preferred = {
        "api_is_fallback": policy["api_is_fallback"],
        "contact_information_recommended": policy["contact_information_recommended"],
        "preferred_sources": policy["preferred_sources"],
    }
    headers = {
        "identifying_user_agent_required": policy[
            "identifying_user_agent_required"
        ]
    }
    retry = {
        "api_max_requests_per_second": policy["api_max_requests_per_second"]
    }
    for values in (preferred, headers, retry):
        values["fingerprint"] = digest(values)
    return {
        "cost": [_row("data-access", preferred)],
        "headers": [_row("data-access", headers)],
        "retry": [_row("data-access", retry)],
    }


def _observed_sources(lock: dict[str, Any], payloads: dict[str, bytes]) -> list[dict[str, Any]]:
    rows = []
    for source in lock["sources"]:
        observed = dict(source)
        observed["sha256"] = hashlib.sha256(payloads[source["id"]]).hexdigest()
        rows.append(observed)
    return rows


def _source_sha256(lock: dict[str, Any], identity: str) -> str:
    matches = [source["sha256"] for source in lock["sources"] if source["id"] == identity]
    if len(matches) != 1:
        raise CratesioAdapterError(f"crates.io source {identity!r} is missing")
    return matches[0]


def validate_stable_cargo_matches(observation: dict[str, Any]) -> None:
    """Reject candidate evidence that breaks a stable Cargo contract."""
    expected = {f"cargo/{contract}" for contract, *_rest in CARGO_CONTRACTS}
    rows = [
        row
        for row in observation["contracts"]["operations"]
        if row["id"].startswith("cargo/")
        and row["values"].get("classification") == "superseded"
    ]
    mismatches = [
        row["id"] for row in rows if row["values"].get("openapi_match") is not True
    ]
    identities = {row["id"] for row in rows}
    if identities != expected or mismatches:
        details = mismatches or sorted(expected.symmetric_difference(identities))
        raise CratesioAdapterError(
            f"stable Cargo contracts no longer match OpenAPI: {details}"
        )


def build_observation(
    lock: dict[str, Any], payloads: dict[str, bytes]
) -> dict[str, Any]:
    """Build one complete crates.io provider-generic observation."""
    if set(payloads) != SOURCE_IDS:
        raise CratesioAdapterError("crates.io drift source set is incomplete")
    try:
        document = parse_json(payloads["openapi"], "crates.io OpenAPI")
        operations = operation_rows(document)
        cargo = cargo_rows(payloads["cargo"])
        policy = policy_observation(
            payloads["policy"],
            payloads["policy-current"],
            _source_sha256(lock, "policy-current"),
        )
        validate_source_evidence(payloads["openapi-source"], payloads["policy-source"])
        operation_contracts, operation_auth, operation_schemas = (
            _operation_contracts(document, operations)
        )
        policies = _policy_contracts(policy)
        contracts = {
            "authentication": [_auth_scheme_contract(document), *operation_auth],
            "cost": policies["cost"],
            "endpoints": [
                _row(
                    "api-v1",
                    {
                        "base_path": "/api/v1",
                        "host": "crates.io",
                        "scheme": "https",
                        "servers_fingerprint": digest(document.get("servers")),
                    },
                )
            ],
            "headers": policies["headers"],
            "idempotency": [],
            "operations": [*operation_contracts, *_cargo_contracts(cargo, operations)],
            "pagination": [],
            "retry": policies["retry"],
            "schemas": [*operation_schemas, *_component_schemas(document)],
        }
    except (KeyError, TypeError, ValueError) as error:
        if isinstance(error, CratesioAdapterError):
            raise
        raise CratesioAdapterError("crates.io source normalization failed") from error
    return {
        "contracts": contracts,
        "format": "cloud-sdk-provider-observation/v1",
        "plugin": dict(lock["plugin"]),
        "provider": lock["provider"],
        "sources": _observed_sources(lock, payloads),
    }
