#!/usr/bin/env python3
"""Build and validate the finite crates.io source-lock observation."""

from __future__ import annotations

import hashlib
import json
import re
from html.parser import HTMLParser
from typing import Any

from cratesio_openapi_auth import synthetic_auth
from cratesio_openapi_parameters import validate_path_parameters
from cratesio_openapi_schema import validate_schema_dialects
from cratesio_reviewed_policy import policy_observation
from cratesio_source_error import SourceLockError

OPENAPI_METHODS = {"delete", "get", "head", "options", "patch", "post", "put", "trace"}
ADMITTED_METHODS = OPENAPI_METHODS - {"trace"}
CLASSIFICATIONS = {"included", "deferred", "excluded", "superseded"}
KNOWN_AUTH = {"api_token", "cookie", "trustpub_token"}
STABLE_OPERATIONS = {
    "add_owners",
    "list_crates",
    "list_owners",
    "publish",
    "remove_owners",
    "unyank_version",
    "yank_version",
}
TSV_COLUMNS = (
    "classification",
    "stability",
    "cargo_contract",
    "method",
    "path",
    "operation_id",
    "observed_auth",
    "admitted_auth",
    "request_media",
    "request_sha256",
    "response_statuses",
    "response_media",
    "response_sha256",
    "policy",
)
CARGO_COLUMNS = (
    "classification",
    "contract",
    "method",
    "path",
    "openapi_operation_id",
    "contract_sha256",
    "policy",
)
CARGO_CONTRACTS = (
    ("publish", "put", "/api/v1/crates/new", "publish", "publish"),
    (
        "yank",
        "delete",
        "/api/v1/crates/{crate_name}/{version}/yank",
        "yank_version",
        "yank",
    ),
    (
        "unyank",
        "put",
        "/api/v1/crates/{crate_name}/{version}/unyank",
        "unyank_version",
        "unyank",
    ),
    (
        "owners-list",
        "get",
        "/api/v1/crates/{crate_name}/owners",
        "list_owners",
        "owners-list",
    ),
    (
        "owners-add",
        "put",
        "/api/v1/crates/{crate_name}/owners",
        "add_owners",
        "owners-add",
    ),
    (
        "owners-remove",
        "delete",
        "/api/v1/crates/{crate_name}/owners",
        "remove_owners",
        "owners-remove",
    ),
    ("search", "get", "/api/v1/crates", "list_crates", "search"),
)

def canonical_bytes(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("ascii")

def digest(value: Any) -> str:
    return hashlib.sha256(canonical_bytes(value)).hexdigest()

def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise SourceLockError(f"JSON contains duplicate key {key!r}")
        result[key] = value
    return result

def parse_json(payload: bytes, label: str) -> dict[str, Any]:
    try:
        value = json.loads(payload, object_pairs_hook=_unique_object)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SourceLockError(f"{label} is not valid UTF-8 JSON") from error
    if not isinstance(value, dict):
        raise SourceLockError(f"{label} root must be an object")
    return value


def _resolve_pointer(document: dict[str, Any], reference: str) -> None:
    if not reference.startswith("#/"):
        raise SourceLockError(f"OpenAPI has external reference {reference!r}")
    current: Any = document
    for raw in reference[2:].split("/"):
        key = raw.replace("~1", "/").replace("~0", "~")
        if not isinstance(current, dict) or key not in current:
            raise SourceLockError(f"OpenAPI has unresolved reference {reference!r}")
        current = current[key]


def _validate_references(document: dict[str, Any]) -> None:
    stack: list[Any] = [document]
    while stack:
        value = stack.pop()
        if isinstance(value, dict):
            reference = value.get("$ref")
            if reference is not None:
                if not isinstance(reference, str):
                    raise SourceLockError("OpenAPI reference must be a string")
                _resolve_pointer(document, reference)
            stack.extend(value.values())
        elif isinstance(value, list):
            stack.extend(value)


def _auth_alternatives(document: dict[str, Any], operation: dict[str, Any]) -> list[str]:
    security = operation.get("security", document.get("security", []))
    if not isinstance(security, list):
        raise SourceLockError("OpenAPI operation security must be an array")
    if not security:
        return ["anonymous"]
    alternatives: list[str] = []
    for entry in security:
        if not isinstance(entry, dict):
            raise SourceLockError("OpenAPI security alternative must be an object")
        names = sorted(entry)
        unknown = set(names) - KNOWN_AUTH
        if unknown:
            raise SourceLockError(f"OpenAPI operation has unknown auth {sorted(unknown)}")
        alternatives.append("+".join(names) if names else "anonymous")
    return alternatives


def _media_types(content: Any) -> str:
    if content is None:
        return "-"
    if not isinstance(content, dict) or any(not isinstance(key, str) for key in content):
        raise SourceLockError("OpenAPI content map is invalid")
    return ",".join(sorted(content)) or "-"


def _response_media(responses: dict[str, Any]) -> str:
    media: set[str] = set()
    for response in responses.values():
        if isinstance(response, dict):
            content = response.get("content", {})
            if isinstance(content, dict):
                media.update(content)
    return ",".join(sorted(media)) or "-"


def _validate_auth_schemes(document: dict[str, Any]) -> None:
    components = document.get("components")
    schemes = components.get("securitySchemes") if isinstance(components, dict) else None
    if not isinstance(schemes, dict) or set(schemes) != KNOWN_AUTH:
        found = sorted(schemes) if isinstance(schemes, dict) else []
        raise SourceLockError(f"OpenAPI auth schemes changed: {found}")
    expected = {
        "api_token": {"type": "apiKey", "in": "header", "name": "authorization"},
        "cookie": {"type": "apiKey", "in": "cookie", "name": "cargo_session"},
        "trustpub_token": {"type": "http", "scheme": "bearer"},
    }
    for name, fields in expected.items():
        scheme = schemes[name]
        if not isinstance(scheme, dict) or any(scheme.get(k) != v for k, v in fields.items()):
            raise SourceLockError(f"OpenAPI auth scheme {name!r} changed")


def operation_rows(document: dict[str, Any]) -> list[dict[str, str]]:
    if document.get("openapi") != "3.1.0":
        raise SourceLockError("OpenAPI version is not exactly 3.1.0")
    validate_schema_dialects(document)
    paths = document.get("paths")
    if not isinstance(paths, dict):
        raise SourceLockError("OpenAPI paths must be an object")
    _validate_auth_schemes(document)
    _validate_references(document)
    rows: list[dict[str, str]] = []
    identities: set[str] = set()
    for path in sorted(paths):
        item = paths[path]
        if not isinstance(path, str) or not isinstance(item, dict):
            raise SourceLockError("OpenAPI path item is invalid")
        present = OPENAPI_METHODS & item.keys()
        unsupported = present - ADMITTED_METHODS
        if unsupported:
            raise SourceLockError(f"{path} exposes unsupported methods: {sorted(unsupported)}")
        for method in sorted(present):
            operation = item[method]
            if not isinstance(operation, dict):
                raise SourceLockError("OpenAPI operation must be an object")
            operation_id = operation.get("operationId")
            if not isinstance(operation_id, str) or not operation_id:
                raise SourceLockError(f"{method.upper()} {path} lacks an operation ID")
            if operation_id in identities:
                raise SourceLockError(f"duplicate operation ID {operation_id!r}")
            identities.add(operation_id)
            validate_path_parameters(
                path,
                item,
                operation,
                operation_id,
                operation_id in STABLE_OPERATIONS,
            )
            responses = operation.get("responses")
            if not isinstance(responses, dict) or not responses:
                raise SourceLockError(f"{operation_id} lacks responses")
            observed = _auth_alternatives(document, operation)
            admitted = [value for value in observed if "cookie" not in value]
            synthetic = synthetic_auth(operation_id, path, item, operation, observed)
            if synthetic is not None:
                admitted = [synthetic]
            if not admitted:
                raise SourceLockError(f"{operation_id} has no non-cookie auth route")
            request = {
                "path_parameters": item.get("parameters", []),
                "parameters": operation.get("parameters", []),
                "request_body": operation.get("requestBody"),
            }
            deprecated = operation.get("deprecated") is True
            stability = (
                "stable-cargo"
                if operation_id in STABLE_OPERATIONS
                else "deprecated-experimental"
                if deprecated
                else "experimental"
            )
            body = operation.get("requestBody")
            content = body.get("content") if isinstance(body, dict) else None
            rows.append(
                {
                    "classification": "included",
                    "stability": stability,
                    "cargo_contract": "yes" if operation_id in STABLE_OPERATIONS else "no",
                    "method": method.upper(),
                    "path": path,
                    "operation_id": operation_id,
                    "observed_auth": "|".join(observed),
                    "admitted_auth": "|".join(admitted),
                    "request_media": _media_types(content),
                    "request_sha256": digest(request),
                    "response_statuses": ",".join(sorted(responses)),
                    "response_media": _response_media(responses),
                    "response_sha256": digest(responses),
                    "policy": "cargo-stable" if operation_id in STABLE_OPERATIONS else "public-openapi",
                }
            )
    return rows


def render_tsv(columns: tuple[str, ...], rows: list[dict[str, str]]) -> bytes:
    lines = ["\t".join(columns)]
    for row in rows:
        if row.get("classification") not in CLASSIFICATIONS:
            raise SourceLockError("source-lock row is unclassified")
        if set(row) != set(columns):
            raise SourceLockError("source-lock row fields are incomplete")
        values = [row[column] for column in columns]
        if any("\t" in value or "\n" in value or not value for value in values):
            raise SourceLockError("source-lock row contains an invalid field")
        lines.append("\t".join(values))
    return ("\n".join(lines) + "\n").encode("ascii")


class _DocumentText(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.parts: list[str] = []
        self.ids: set[str] = set()
        self.duplicate_id = False

    def handle_starttag(self, _tag: str, attrs: list[tuple[str, str | None]]) -> None:
        for name, value in attrs:
            if name == "id" and value:
                if value in self.ids:
                    self.duplicate_id = True
                self.ids.add(value)

    def handle_data(self, data: str) -> None:
        self.parts.append(data)


def html_text(payload: bytes, label: str) -> tuple[str, set[str]]:
    try:
        source = payload.decode("utf-8")
    except UnicodeDecodeError as error:
        raise SourceLockError(f"{label} is not UTF-8 HTML") from error
    if "<html" not in source.lower() or "</html>" not in source.lower():
        raise SourceLockError(f"{label} is incomplete HTML")
    parser = _DocumentText()
    try:
        parser.feed(source)
        parser.close()
    except Exception as error:
        raise SourceLockError(f"{label} is malformed HTML") from error
    if parser.duplicate_id:
        raise SourceLockError(f"{label} has duplicate element IDs")
    return " ".join(" ".join(parser.parts).split()), parser.ids


def cargo_rows(payload: bytes) -> list[dict[str, str]]:
    text, ids = html_text(payload, "Cargo Registry Web API")
    source = payload.decode("utf-8")
    rows: list[dict[str, str]] = []
    for contract, method, path, operation_id, section in CARGO_CONTRACTS:
        marker = re.search(rf'<h([23])\s+id="{re.escape(section)}"[^>]*>', source)
        if marker is None:
            raise SourceLockError(f"Cargo contract {contract!r} is missing")
        next_heading = re.search(
            r"<h2\s+id=|<h3\s+id=" if marker.group(1) == "3" else r"<h2\s+id=",
            source[marker.end() :],
        )
        end = marker.end() + next_heading.start() if next_heading else len(source)
        fragment = _DocumentText()
        fragment.feed(source[marker.end() : end])
        section_text = " ".join(" ".join(fragment.parts).split())
        if path not in section_text or f"Method: {method.upper()}" not in section_text:
            raise SourceLockError(f"Cargo contract {contract!r} changed")
        rows.append(
            {
                "classification": "superseded",
                "contract": contract,
                "method": method.upper(),
                "path": path,
                "openapi_operation_id": operation_id,
                "contract_sha256": digest(section_text),
                "policy": "implemented-through-public-openapi-row",
            }
        )
    if "login" not in ids or "/me" not in text:
        raise SourceLockError("Cargo login instruction target is missing")
    rows.append(
        {
            "classification": "excluded",
            "contract": "login-instruction",
            "method": "INSTRUCTION",
            "path": "/me",
            "openapi_operation_id": "-",
            "contract_sha256": digest("/me login instruction target"),
            "policy": "not-an-api-operation",
        }
    )
    return rows


def validate_source_evidence(openapi_source: bytes, policy_source: bytes) -> None:
    try:
        openapi_text = openapi_source.decode("utf-8")
        policy_text = policy_source.decode("utf-8")
    except UnicodeDecodeError as error:
        raise SourceLockError("official source evidence is not UTF-8") from error
    if "crates.io data access policy" not in openapi_text or "OpenApi" not in openapi_text:
        raise SourceLockError("OpenAPI implementation evidence is incomplete")
    if "Data Access Policy" not in policy_text or "A maximum of 1 request per second" not in policy_text:
        raise SourceLockError("policy implementation evidence is incomplete")


def validate_tsv(data: bytes, columns: tuple[str, ...], expected_rows: int) -> list[dict[str, str]]:
    try:
        text = data.decode("ascii")
    except UnicodeDecodeError as error:
        raise SourceLockError("crates.io TSV evidence is not ASCII") from error
    if not text.endswith("\n"):
        raise SourceLockError("crates.io TSV evidence lacks a terminal newline")
    lines = text.splitlines()
    if not lines or tuple(lines[0].split("\t")) != columns:
        raise SourceLockError("crates.io TSV header changed")
    if len(lines) != expected_rows + 1:
        raise SourceLockError("crates.io TSV row count changed")
    rows: list[dict[str, str]] = []
    identities: set[str] = set()
    for line in lines[1:]:
        values = line.split("\t")
        if len(values) != len(columns):
            raise SourceLockError("crates.io TSV row is incomplete")
        row = dict(zip(columns, values, strict=True))
        if row["classification"] not in CLASSIFICATIONS or any(not value for value in values):
            raise SourceLockError("crates.io TSV row is unclassified or empty")
        identity = row.get("operation_id", row.get("contract", ""))
        if identity in identities:
            raise SourceLockError("crates.io TSV contains a duplicate identity")
        identities.add(identity)
        rows.append(row)
    return rows


def observe(lock: dict[str, Any], payloads: dict[str, bytes]) -> tuple[bytes, bytes, dict[str, Any]]:
    document = parse_json(payloads["openapi"], "crates.io OpenAPI")
    operations = operation_rows(document)
    cargo = cargo_rows(payloads["cargo"])
    policy_sha256 = next(
        source["sha256"]
        for source in lock["sources"]
        if source["id"] == "policy-current"
    )
    policy = policy_observation(
        payloads["policy"], payloads["policy-current"], policy_sha256
    )
    validate_source_evidence(payloads["openapi-source"], payloads["policy-source"])
    summary = {
        "openapi": {
            "version": document["openapi"],
            "paths": len(document["paths"]),
            "operations": len(operations),
            "auth_schemes": sorted(document["components"]["securitySchemes"]),
        },
        "cargo": {
            "stable_operations": sum(row["classification"] == "superseded" for row in cargo),
            "instruction_targets": sum(row["classification"] == "excluded" for row in cargo),
        },
        "policy": policy,
    }
    return render_tsv(TSV_COLUMNS, operations), render_tsv(CARGO_COLUMNS, cargo), summary
