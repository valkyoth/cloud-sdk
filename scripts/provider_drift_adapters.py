#!/usr/bin/env python3
"""Repository-reviewed adapters from authenticated sources to observations."""

from __future__ import annotations

import csv
import hashlib
import io
from pathlib import Path
from typing import Any, Callable
from urllib.parse import urlsplit

import check_hetzner_api_drift as hetzner
import generate_response_operations as responses
from provider_drift_model import read_bounded_bytes


ROOT = Path(__file__).resolve().parents[1]
MAX_POLICY_BYTES = 16 * 1024 * 1024


class AdapterError(RuntimeError):
    """Authenticated source bytes could not produce a valid observation."""


def _render_dict_rows(rows: list[dict[str, str]], fields: list[str]) -> bytes:
    output = io.StringIO(newline="")
    writer = csv.DictWriter(
        output, delimiter="\t", fieldnames=fields, lineterminator="\n"
    )
    writer.writeheader()
    writer.writerows(rows)
    return output.getvalue().encode("utf-8")


def _render_response_rows(rows: list[tuple[str, ...]]) -> bytes:
    operation_ids = [row[1] for row in rows]
    if len(set(operation_ids)) != len(operation_ids):
        raise AdapterError("provider response operations are not unique")
    for row in rows:
        for index, value in enumerate(row):
            try:
                responses.validate_tsv_cell(value, f"response field {index}")
            except ValueError as error:
                raise AdapterError("provider response operation is invalid") from error
    lines = ["api\toperation_id\tstatus\tshape\troot\trequired"]
    lines.extend("\t".join(row) for row in sorted(rows, key=lambda row: row[1]))
    return ("\n".join(lines) + "\n").encode("ascii")


def _evidence(payload: bytes, count: int, path: str) -> dict[str, Any]:
    return {
        "count": count,
        "path": path,
        "sha256": hashlib.sha256(payload).hexdigest(),
    }


def _local_evidence(path: str) -> dict[str, str]:
    try:
        payload = read_bounded_bytes(
            ROOT / path, "provider policy evidence", MAX_POLICY_BYTES
        )
    except ValueError as error:
        raise AdapterError("provider policy evidence is unavailable") from error
    return {"path": path, "sha256": hashlib.sha256(payload).hexdigest()}


def _authentication_row(service: str, document: dict[str, Any]) -> dict[str, Any]:
    security = document.get("security")
    schemes = document.get("components", {}).get("securitySchemes")
    if (
        not isinstance(security, list)
        or len(security) != 1
        or not isinstance(security[0], dict)
        or len(security[0]) != 1
        or not isinstance(schemes, dict)
    ):
        raise AdapterError("provider authentication structure is invalid")
    source_name, scopes = next(iter(security[0].items()))
    scheme = schemes.get(source_name)
    if (
        not isinstance(source_name, str)
        or not isinstance(scopes, list)
        or not all(isinstance(scope, str) for scope in scopes)
        or not isinstance(scheme, dict)
        or not isinstance(scheme.get("type"), str)
        or not isinstance(scheme.get("scheme"), str)
    ):
        raise AdapterError("provider authentication structure is invalid")
    return {
        "id": f"{service}-bearer",
        "values": {
            "endpoint_scope": "exact",
            "scheme": scheme["scheme"],
            "service": service,
            "source_name": source_name,
            "source_sha256": hetzner.digest(
                {"security": security, "securitySchemes": schemes}
            ),
            "type": scheme["type"],
        },
    }


def _endpoint_row(service: str, document: dict[str, Any]) -> dict[str, Any]:
    servers = document.get("servers")
    if (
        not isinstance(servers, list)
        or len(servers) != 1
        or not isinstance(servers[0], dict)
        or not isinstance(servers[0].get("url"), str)
    ):
        raise AdapterError("provider endpoint structure is invalid")
    parsed = urlsplit(servers[0]["url"])
    if (
        parsed.scheme != "https"
        or not parsed.hostname
        or parsed.query
        or parsed.fragment
        or parsed.username is not None
        or parsed.password is not None
    ):
        raise AdapterError("provider endpoint structure is invalid")
    try:
        port = parsed.port or 443
    except ValueError as error:
        raise AdapterError("provider endpoint structure is invalid") from error
    return {
        "id": f"{service}-v1",
        "values": {
            "base_path": parsed.path,
            "host": parsed.hostname,
            "port": port,
            "scheme": parsed.scheme,
            "source_sha256": hetzner.digest(servers),
        },
    }


def _header_row(service: str, document: dict[str, Any]) -> dict[str, Any]:
    records: list[dict[str, Any]] = []
    names: set[str] = set()
    paths = document.get("paths")
    if not isinstance(paths, dict):
        raise AdapterError("provider path structure is invalid")
    for path, item in paths.items():
        if not isinstance(path, str) or not isinstance(item, dict):
            raise AdapterError("provider path structure is invalid")
        for method, operation in item.items():
            if method not in hetzner.HTTP_METHODS:
                continue
            if not isinstance(operation, dict):
                raise AdapterError("provider operation structure is invalid")
            response_headers: dict[str, Any] = {}
            responses_value = operation.get("responses", {})
            if not isinstance(responses_value, dict):
                raise AdapterError("provider response structure is invalid")
            for status, response in responses_value.items():
                if not isinstance(status, str) or not isinstance(response, dict):
                    raise AdapterError("provider response structure is invalid")
                headers = response.get("headers", {})
                if not isinstance(headers, dict) or any(
                    not isinstance(name, str) for name in headers
                ):
                    raise AdapterError("provider response headers are invalid")
                names.update(name.lower() for name in headers)
                if headers:
                    response_headers[status] = headers
            if response_headers:
                records.append(
                    {
                        "headers": response_headers,
                        "method": method,
                        "path": path,
                    }
                )
    return {
        "id": f"{service}-response-headers",
        "values": {
            "names": sorted(names),
            "source_sha256": hetzner.digest(records),
        },
    }


def _pagination_row(
    api: str, service: str, operations: list[dict[str, str]]
) -> dict[str, Any]:
    operation_ids = sorted(
        row["operation_id"]
        for row in operations
        if row["api"] == api and row["pagination"] == "yes"
    )
    return {
        "id": f"{service}-numbered",
        "values": {
            "operation_count": len(operation_ids),
            "operations_sha256": hetzner.digest(operation_ids),
            "page": "page",
            "per_page": "per_page",
            "provider_links": "same_authority",
        },
    }


def _policy_contracts() -> dict[str, list[dict[str, Any]]]:
    associations = _local_evidence("docs/OPERATION_ASSOCIATIONS.tsv")
    return {
        "cost": [
            {
                "id": "operation-policy",
                "values": {**associations, "mutation_authority": "required"},
            }
        ],
        "idempotency": [
            {"id": "operation-policy", "values": dict(associations)}
        ],
        "retry": [
            {
                "id": "operation-policy",
                "values": {
                    **associations,
                    "delivery_phases": [
                        "not_sent",
                        "possibly_sent",
                        "response_started",
                    ],
                },
            }
        ],
    }


def _hetzner_observation(
    lock: dict[str, Any], payloads: dict[str, bytes]
) -> dict[str, Any]:
    if set(payloads) != {"cloud-openapi", "dns-openapi"}:
        raise AdapterError("Hetzner source set is incomplete")
    try:
        documents = {
            "cloud": hetzner.parse_spec("cloud", payloads["cloud-openapi"]),
            "hetzner": hetzner.parse_spec("hetzner", payloads["dns-openapi"]),
        }
        operations: list[dict[str, str]] = []
        schemas: list[dict[str, str]] = []
        response_rows: list[tuple[str, ...]] = []
        for api, document in documents.items():
            operations.extend(hetzner.operation_rows(api, document))
            schemas.extend(hetzner.schema_rows(api, document))
            response_rows.extend(responses.rows(api, document))
    except (KeyError, SystemExit, ValueError) as error:
        raise AdapterError("Hetzner source normalization failed") from error

    operation_payload = _render_dict_rows(
        sorted(operations, key=lambda row: (row["api"], row["path"], row["method"])),
        [
            "api",
            "method",
            "path",
            "tag",
            "operation_id",
            "deprecated",
            "pagination",
            "sorting",
            "action",
            "fingerprint",
        ],
    )
    schema_payload = _render_dict_rows(
        sorted(schemas, key=lambda row: (row["api"], row["schema"])),
        ["api", "schema", "fingerprint"],
    )
    response_payload = _render_response_rows(response_rows)

    policies = _policy_contracts()
    contracts = {
        "authentication": [
            _authentication_row("cloud", documents["cloud"]),
            _authentication_row("dns", documents["hetzner"]),
        ],
        "cost": policies["cost"],
        "endpoints": [
            _endpoint_row("cloud", documents["cloud"]),
            _endpoint_row("dns", documents["hetzner"]),
        ],
        "headers": [
            _header_row("cloud", documents["cloud"]),
            _header_row("dns", documents["hetzner"]),
            {
                "id": "response-metadata-policy",
                "values": {
                    "duplicate_policy": "reject",
                    "evidence": [
                        _local_evidence(
                            "crates/cloud-sdk-hetzner/src/prepared/wire_policy.rs"
                        ),
                        _local_evidence(
                            "crates/cloud-sdk-hetzner/src/prepared/operation.rs"
                        ),
                    ],
                    "request_id": "operation_policy",
                    "unknown_sensitivity": "sensitive",
                },
            },
            {
                "id": "rate-limit-policy",
                "values": {
                    "evidence": _local_evidence(
                        "crates/cloud-sdk-hetzner/src/rate_limit.rs"
                    ),
                    "fields": [
                        "ratelimit-limit",
                        "ratelimit-remaining",
                        "ratelimit-reset",
                    ],
                    "presence": "all_or_none",
                },
            },
        ],
        "idempotency": policies["idempotency"],
        "operations": [],
        "pagination": [
            _pagination_row("cloud", "cloud", operations),
            _pagination_row("hetzner", "dns", operations),
        ],
        "retry": policies["retry"],
        "schemas": [],
    }
    active = sum(row["deprecated"] == "no" for row in operations)
    contracts["operations"] = [
        {
            "id": "active-operation-lock",
            "values": {
                **_evidence(
                    operation_payload, len(operations), "docs/API_FINGERPRINTS.tsv"
                ),
                "active_count": active,
            },
        },
        {
            "id": "response-binding-lock",
            "values": _evidence(
                response_payload,
                len(response_rows),
                "crates/cloud-sdk-hetzner/src/serde/response_operations.tsv",
            ),
        },
    ]
    contracts["schemas"] = [
        {
            "id": "openapi-schema-lock",
            "values": _evidence(
                schema_payload, len(schemas), "docs/API_SCHEMA_FINGERPRINTS.tsv"
            ),
        }
    ]
    observation = {
        "contracts": contracts,
        "format": "cloud-sdk-provider-observation/v1",
        "plugin": dict(lock["plugin"]),
        "provider": lock["provider"],
        "sources": [dict(source) for source in lock["sources"]],
    }
    source_payloads = {
        "cloud-openapi": payloads["cloud-openapi"],
        "dns-openapi": payloads["dns-openapi"],
    }
    for source in observation["sources"]:
        source["sha256"] = hashlib.sha256(source_payloads[source["id"]]).hexdigest()

    return observation


Adapter = Callable[[dict[str, Any], dict[str, bytes]], dict[str, Any]]
ADAPTERS: dict[tuple[str, str, int], Adapter] = {
    ("hetzner", "normalized-json", 1): _hetzner_observation,
}


def build_live_observation(
    lock: dict[str, Any], payloads: dict[str, bytes]
) -> dict[str, Any]:
    key = (lock["provider"], lock["plugin"]["id"], lock["plugin"]["version"])
    adapter = ADAPTERS.get(key)
    if adapter is None:
        raise AdapterError("provider and plugin have no reviewed source adapter")
    try:
        return adapter(lock, payloads)
    except AdapterError:
        raise
    except (
        csv.Error,
        KeyError,
        OverflowError,
        RecursionError,
        TypeError,
        UnicodeError,
        ValueError,
    ) as error:
        raise AdapterError("provider source normalization failed") from error
