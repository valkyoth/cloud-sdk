#!/usr/bin/env python3
"""Repository-reviewed adapters from authenticated sources to observations."""

from __future__ import annotations

import copy
import csv
import hashlib
import io
from typing import Any, Callable

import check_hetzner_api_drift as hetzner
import generate_response_operations as responses


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


def _rows(contracts: dict[str, Any], category: str) -> dict[str, dict[str, Any]]:
    return {row["id"]: row["values"] for row in contracts[category]}


def _evidence(payload: bytes, count: int, path: str) -> dict[str, Any]:
    return {
        "count": count,
        "path": path,
        "sha256": hashlib.sha256(payload).hexdigest(),
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

    observation = {
        "contracts": copy.deepcopy(lock["contracts"]),
        "format": "cloud-sdk-provider-observation/v1",
        "plugin": copy.deepcopy(lock["plugin"]),
        "provider": lock["provider"],
        "sources": copy.deepcopy(lock["sources"]),
    }
    source_payloads = {
        "cloud-openapi": payloads["cloud-openapi"],
        "dns-openapi": payloads["dns-openapi"],
    }
    for source in observation["sources"]:
        source["sha256"] = hashlib.sha256(source_payloads[source["id"]]).hexdigest()

    try:
        operation_values = _rows(observation["contracts"], "operations")
        active = sum(row["deprecated"] == "no" for row in operations)
        operation_values["active-operation-lock"].update(
            _evidence(operation_payload, len(operations), "docs/API_FINGERPRINTS.tsv")
        )
        operation_values["active-operation-lock"]["active_count"] = active
        operation_values["response-binding-lock"].update(
            _evidence(
                response_payload,
                len(response_rows),
                "crates/cloud-sdk-hetzner/src/serde/response_operations.tsv",
            )
        )
        schema_values = _rows(observation["contracts"], "schemas")
        schema_values["openapi-schema-lock"].update(
            _evidence(schema_payload, len(schemas), "docs/API_SCHEMA_FINGERPRINTS.tsv")
        )
    except KeyError as error:
        raise AdapterError("Hetzner lock lacks required evidence rows") from error
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
    return adapter(lock, payloads)
