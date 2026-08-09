#!/usr/bin/env python3
"""Generate exhaustive compile-time Hetzner operation associations."""

from __future__ import annotations

import argparse
import csv
import json
import subprocess
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FINGERPRINTS = ROOT / "docs" / "API_FINGERPRINTS.tsv"
BODIES = ROOT / "docs" / "PREPARED_BODY_OPERATIONS.txt"
RESPONSES = (
    ROOT / "crates" / "cloud-sdk-hetzner" / "src" / "serde" / "response_operations.tsv"
)
ASSOCIATIONS = ROOT / "docs" / "OPERATION_ASSOCIATIONS.tsv"
RESPONSE_IDENTITIES = ROOT / "docs" / "RESPONSE_IDENTITY_CLASSES.tsv"
OUTPUT = ROOT / "crates" / "cloud-sdk-hetzner" / "src" / "association" / "markers.rs"
PROVIDER_LOCK = ROOT / "provider-drift" / "providers" / "hetzner.lock.json"
EXPECTED_OPERATIONS = 208
RESPONSE_IDENTITY_COLUMNS = ("operation_id", "response_identity")

ASSOCIATION_COLUMNS = (
    "operation_id",
    "service",
    "authentication",
    "query_policy",
    "body_policy",
    "retry_policy",
    "permit_class",
)

METHOD_TYPES = {
    "GET": "GetMethod",
    "POST": "PostMethod",
    "PUT": "PutMethod",
    "DELETE": "DeleteMethod",
}
STATUS_TYPES = {"200": "StatusOk", "201": "StatusCreated", "204": "StatusNoContent"}
RESPONSE_TYPES = {
    "empty": "EmptyResponse",
    "action": "ActionResponse",
    "actions": "ActionsResponse",
    "actions-page": "ActionsPageResponse",
    "resource": "ResourceResponse",
    "resource-list": "ResourceListResponse",
    "resource-page": "ResourcePageResponse",
    "composite": "CompositeResponse",
    "metrics": "MetricsResponse",
    "zonefile": "ZoneFileResponse",
    "pricing": "PricingResponse",
    "folders": "FoldersResponse",
}
SERVICE_TYPES = {
    "cloud": ("CloudService", "CloudEndpointPolicy"),
    "dns": ("DnsService", "CloudEndpointPolicy"),
    "security": ("SecurityService", "CloudEndpointPolicy"),
    "storage": ("StorageService", "StorageEndpointPolicy"),
}
AUTHENTICATION_TYPES = {
    "bearer": "BearerAuthentication",
    "basic": "BasicAuthentication",
}
QUERY_TYPES = {
    "forbidden": "QueryForbidden",
    "optional": "OptionalQuery",
    "required": "RequiredQuery",
}
BODY_TYPES = {"forbidden": "BodyForbidden", "json": "JsonBody"}
RETRY_TYPES = {"never": "NeverRetry", "explicit": "ExplicitRetry"}
PERMIT_TYPES = {
    "none": "NoPermit",
    "mutation": "MutationPermit",
    "destructive": "DestructivePermit",
    "cost": "CostPermit",
}
RESPONSE_IDENTITY_TYPES = {
    "none": "ResponseIdentityClass::None",
    "exact-resource": "ResponseIdentityClass::ExactResource",
    "parent-resource": "ResponseIdentityClass::ParentResource",
}


@dataclass(frozen=True)
class Operation:
    api: str
    method: str
    path: str
    tag: str
    operation_id: str
    pagination: str
    status: str
    response: str
    response_root: str
    response_required: str
    service: str
    authentication: str
    query_policy: str
    body_policy: str
    retry_policy: str
    permit_class: str
    response_identity: str


def read_response_identities(path: Path = RESPONSE_IDENTITIES) -> dict[str, str]:
    """Read explicit non-default response identity classes."""
    with path.open(encoding="ascii", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if tuple(reader.fieldnames or ()) != RESPONSE_IDENTITY_COLUMNS:
            raise ValueError("response identity source has an invalid schema")
        rows = list(reader)
    identities: dict[str, str] = {}
    for number, row in enumerate(rows, 2):
        operation_id = row.get("operation_id", "")
        identity = row.get("response_identity", "")
        if not operation_id or identity not in RESPONSE_IDENTITY_TYPES or identity == "none":
            raise ValueError(f"invalid response identity row {number}")
        if operation_id in identities:
            raise ValueError("response identity source has duplicate operation IDs")
        identities[operation_id] = identity
    if list(identities) != sorted(identities):
        raise ValueError("response identity rows are not sorted")
    return identities


def read_tsv(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="ascii", newline="") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


def read_associations(path: Path = ASSOCIATIONS) -> list[dict[str, str]]:
    with path.open(encoding="ascii", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if tuple(reader.fieldnames or ()) != ASSOCIATION_COLUMNS:
            raise ValueError("operation association source has an invalid schema")
        rows = list(reader)
    for number, row in enumerate(rows, 2):
        if None in row or any(not row[column] for column in ASSOCIATION_COLUMNS):
            raise ValueError(f"invalid operation association row {number}")
        validate_value(row, "service", SERVICE_TYPES, number)
        validate_value(row, "authentication", AUTHENTICATION_TYPES, number)
        validate_value(row, "query_policy", QUERY_TYPES, number)
        validate_value(row, "body_policy", BODY_TYPES, number)
        validate_value(row, "retry_policy", RETRY_TYPES, number)
        validate_value(row, "permit_class", PERMIT_TYPES, number)
    operation_ids = [row["operation_id"] for row in rows]
    if operation_ids != sorted(operation_ids):
        raise ValueError("operation association rows are not sorted")
    if len(operation_ids) != len(set(operation_ids)):
        raise ValueError("operation association source has duplicate operation IDs")
    if any(
        not operation_id
        or not operation_id[0].islower()
        or not all(
            character.islower() or character.isdigit() or character == "_"
            for character in operation_id
        )
        for operation_id in operation_ids
    ):
        raise ValueError("operation association source has an invalid operation ID")
    return rows


def validate_value(
    row: dict[str, str], column: str, admitted: dict[str, object], number: int
) -> None:
    if row[column] not in admitted:
        raise ValueError(f"unknown {column} at operation association row {number}")


def read_bodies(path: Path) -> set[str]:
    values = {
        line.strip()
        for line in path.read_text(encoding="ascii").splitlines()
        if line.strip() and not line.startswith("#")
    }
    if len(values) != 91:
        raise ValueError("request-body operation count changed")
    return values


def indexed(rows: list[dict[str, str]], key: str, source: str) -> dict[str, dict[str, str]]:
    result: dict[str, dict[str, str]] = {}
    for row in rows:
        value = row[key]
        if value in result:
            raise ValueError(f"duplicate {source} operation: {value}")
        result[value] = row
    return result


def source_authentication() -> dict[str, str]:
    try:
        lock = json.loads(PROVIDER_LOCK.read_text(encoding="utf-8"))
        rows = lock["contracts"]["authentication"]
    except (OSError, UnicodeError, json.JSONDecodeError, KeyError, TypeError) as error:
        raise ValueError("provider authentication lock is unavailable") from error
    schemes: dict[str, str] = {}
    for row in rows:
        try:
            values = row["values"]
            service = values["service"]
            scheme = values["scheme"]
        except (KeyError, TypeError) as error:
            raise ValueError("provider authentication lock is invalid") from error
        if service in schemes or scheme not in AUTHENTICATION_TYPES:
            raise ValueError("provider authentication lock is invalid")
        schemes[service] = scheme
    if set(schemes) != {"cloud", "storage"}:
        raise ValueError("provider authentication lock has the wrong services")
    return {"cloud": schemes["cloud"], "hetzner": schemes["storage"]}


def load_operations() -> list[Operation]:
    fingerprint_rows = [
        row for row in read_tsv(FINGERPRINTS) if row["deprecated"] == "no"
    ]
    fingerprints = indexed(fingerprint_rows, "operation_id", "fingerprint")
    responses = indexed(read_tsv(RESPONSES), "operation_id", "response")
    associations = indexed(
        read_associations(), "operation_id", "operation association"
    )
    bodies = read_bodies(BODIES)
    response_identities = read_response_identities()
    authentication = source_authentication()
    active = set(fingerprints)
    if len(active) != EXPECTED_OPERATIONS:
        raise ValueError("active operation count changed")
    if set(responses) != active:
        raise ValueError("response bindings do not exactly cover active operations")
    if set(associations) != active:
        raise ValueError("operation associations do not exactly cover active operations")
    if not bodies <= active:
        raise ValueError("request-body lock contains inactive operations")
    if not set(response_identities) <= active:
        raise ValueError("response identity source contains inactive operations")
    operations = []
    for operation_id in sorted(active):
        source = fingerprints[operation_id]
        response = responses[operation_id]
        association = associations[operation_id]
        if source["api"] != response["api"]:
            raise ValueError(f"API source mismatch for {operation_id}")
        if association["service"] != response["service"]:
            raise ValueError(f"response service mismatch for {operation_id}")
        body_present = operation_id in bodies
        if (association["body_policy"] == "json") != body_present:
            raise ValueError(f"body source mismatch for {operation_id}")
        if (association["service"] == "storage") != (source["api"] == "hetzner"):
            raise ValueError(f"service source mismatch for {operation_id}")
        if association["authentication"] != authentication[source["api"]]:
            raise ValueError(f"authentication source mismatch for {operation_id}")
        operations.append(
            Operation(
                api=source["api"],
                method=source["method"],
                path=source["path"],
                tag=source["tag"],
                operation_id=operation_id,
                pagination=source["pagination"],
                status=response["status"],
                response=response["shape"],
                response_root=response["root"],
                response_required=response["required"],
                service=association["service"],
                authentication=association["authentication"],
                query_policy=association["query_policy"],
                body_policy=association["body_policy"],
                retry_policy=association["retry_policy"],
                permit_class=association["permit_class"],
                response_identity=response_identities.get(operation_id, "none"),
            )
        )
    return operations


def pascal(value: str) -> str:
    return "".join(part[:1].upper() + part[1:] for part in value.split("_"))


def row(operation: Operation) -> str:
    service_type, endpoint = SERVICE_TYPES[operation.service]
    authentication = AUTHENTICATION_TYPES[operation.authentication]
    body = BODY_TYPES[operation.body_policy]
    values = (
        pascal(operation.operation_id),
        f'"{operation.operation_id}"',
        service_type,
        endpoint,
        authentication,
        METHOD_TYPES[operation.method],
        QUERY_TYPES[operation.query_policy],
        body,
        STATUS_TYPES[operation.status],
        RESPONSE_TYPES[operation.response],
        json.dumps(operation.path),
        json.dumps(operation.response_root),
        json.dumps(operation.response_required),
        RESPONSE_IDENTITY_TYPES[operation.response_identity],
        "NumberedPagination" if operation.pagination == "yes" else "NoPagination",
        RETRY_TYPES[operation.retry_policy],
        PERMIT_TYPES[operation.permit_class],
    )
    return "        (" + ", ".join(values) + "),"


def render() -> str:
    operations = load_operations()
    rows = "\n".join(row(operation) for operation in operations)
    return f'''//! Generated exhaustive active-operation associations.
//!
//! Regenerate with `scripts/generate_operation_associations.py`.

use cloud_sdk::{{ServiceMarker, operation_id}};

use super::policy::{{
    HetznerOperation, OperationDescriptor, ReadOnlyOperation, ResponseIdentityClass, Sealed,
}};
use super::OperationBindingEvidence;
use super::types::*;
use crate::identity::{{CloudService, DnsService, SecurityService, StorageService}};

macro_rules! body_headers {{
    (BodyForbidden) => {{ AcceptJson }};
    (JsonBody) => {{ AcceptAndContentTypeJson }};
}}
macro_rules! body_media {{
    (BodyForbidden) => {{ NoRequestMedia }};
    (JsonBody) => {{ JsonRequestMedia }};
}}
macro_rules! success_body {{
    (EmptyResponse) => {{ EmptySuccessBody }};
    ($response:ident) => {{ JsonSuccessBody }};
}}
macro_rules! success_media {{
    (EmptyResponse) => {{ ForbiddenSuccessMedia }};
    ($response:ident) => {{ JsonSuccessMedia }};
}}

macro_rules! read_only_operation {{
    ($marker:ident, NoPermit) => {{
        impl ReadOnlyOperation for $marker {{}}
    }};
    ($marker:ident, $permit:ident) => {{}};
}}

macro_rules! operation_associations {{
    ($(($marker:ident, $id:literal, $service:ident, $endpoint:ident, $authentication:ident,
        $method:ident, $query:ident, $body:ident, $status:ident, $response:ident,
        $path:literal, $success_root:literal, $success_required:literal, $response_identity:expr,
        $pagination:ident, $retry:ident, $permit:ident),)+) => {{
        /// Sealed markers for every active source-locked Hetzner operation.
        pub mod operations {{
            use super::*;
            $(
                #[doc = concat!("Association for Hetzner operation `", $id, "`.")]
                #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
                pub struct $marker;
                impl Sealed for $marker {{}}
                impl HetznerOperation for $marker {{
                    type Service = $service;
                    type EndpointPolicy = $endpoint;
                    type Authentication = $authentication;
                    type AuthenticationScope = RequiredServiceScope;
                    type Query = $query;
                    type Body = $body;
                    type RequestHeaders = body_headers!($body);
                    type RequestMedia = body_media!($body);
                    type SuccessStatus = $status;
                    type SuccessBody = success_body!($response);
                    type SuccessMedia = success_media!($response);
                    type ErrorBody = JsonErrorBody;
                    type ErrorMedia = JsonErrorMedia;
                    type ResponseCaps = JsonResponseCaps;
                    type Pagination = $pagination;
                    type Quota = HetznerQuota;
                    type Retry = $retry;
                    type Streaming = BufferedStreaming;
                    type Success = $response;
                    type Error = HetznerErrorResponse;
                    type Permit = $permit;
                    const DESCRIPTOR: OperationDescriptor = OperationDescriptor::new(
                        operation_id!($id),
                        <$service as ServiceMarker>::ID,
                        <$endpoint as EndpointAssociation>::BASE,
                        <$authentication as AuthenticationAssociation>::CLASS,
                        <$method as MethodAssociation>::METHOD,
                        <$query as QueryAssociation>::POLICY,
                        <$body as BodyAssociation>::POLICY,
                        <$status as StatusAssociation>::STATUS,
                        <$response as ResponseAssociation>::SHAPE,
                        $path,
                        $success_root,
                        $success_required,
                        $response_identity,
                        <$pagination as PaginationAssociation>::POLICY,
                        <$retry as RetryAssociation>::POLICY,
                        <$permit as PermitAssociation>::CLASS,
                    );
                }}
                read_only_operation!($marker, $permit);
            )+
        }}

        /// Descriptors for all active operations in stable operation-ID order.
        pub const ALL_OPERATIONS: &[OperationDescriptor] = &[
            $(<operations::$marker as HetznerOperation>::DESCRIPTOR,)+
        ];

        /// Marker-derived evidence for all active operations in stable order.
        #[doc(hidden)]
        pub const ALL_OPERATION_EVIDENCE: &[OperationBindingEvidence] = &[
            $(OperationBindingEvidence::of::<operations::$marker>(),)+
        ];

        pub(crate) fn operation_path_template(operation_id: &str) -> Option<&'static str> {{
            match operation_id {{
                $($id => Some($path),)+
                _ => None,
            }}
        }}
    }};
}}

#[rustfmt::skip]
operation_associations!(
{rows}
);
'''


def formatted_render() -> str:
    result = subprocess.run(
        ["rustfmt", "--edition", "2024", "--emit", "stdout"],
        input=render(),
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise SystemExit("rustfmt failed while generating operation associations")
    return result.stdout


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    generated = formatted_render()
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text(encoding="ascii") != generated:
            raise SystemExit("operation associations are stale; regenerate them")
        print(f"{EXPECTED_OPERATIONS} operation associations are current.")
        return 0
    OUTPUT.write_text(generated, encoding="ascii")
    print(f"generated {EXPECTED_OPERATIONS} operation associations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
