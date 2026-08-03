#!/usr/bin/env python3
"""Generate exhaustive compile-time Hetzner operation associations."""

from __future__ import annotations

import argparse
import csv
import subprocess
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FINGERPRINTS = ROOT / "docs" / "API_FINGERPRINTS.tsv"
BODIES = ROOT / "docs" / "PREPARED_BODY_OPERATIONS.txt"
RESPONSES = (
    ROOT / "crates" / "cloud-sdk-hetzner" / "src" / "serde" / "response_operations.tsv"
)
OUTPUT = ROOT / "crates" / "cloud-sdk-hetzner" / "src" / "association" / "markers.rs"
EXPECTED_OPERATIONS = 208

DNS_TAGS = {"Zones", "Zone Actions", "Zone RRSets", "Zone RRSet Actions"}
SECURITY_TAGS = {"Certificates", "Certificate Actions", "SSH Keys"}
REQUIRED_QUERY = {"get_actions", "get_load_balancer_metrics", "get_server_metrics"}
OPTIONAL_QUERY_WITHOUT_PAGINATION = {
    "list_storage_box_snapshots",
    "list_storage_box_subaccounts",
}
COST_OPERATIONS = {
    "change_load_balancer_type",
    "change_server_type",
    "change_storage_box_type",
    "create_floating_ip",
    "create_load_balancer",
    "create_primary_ip",
    "create_server",
    "create_server_image",
    "create_storage_box",
    "create_volume",
    "enable_server_backup",
    "resize_volume",
}
DESTRUCTIVE_POST_OPERATIONS = {
    "change_floating_ip_protection",
    "change_image_protection",
    "change_load_balancer_protection",
    "change_network_protection",
    "change_primary_ip_protection",
    "change_server_protection",
    "change_storage_box_protection",
    "change_volume_protection",
    "change_zone_protection",
    "change_zone_rrset_protection",
    "delete_load_balancer_service",
    "delete_network_route",
    "delete_network_subnet",
    "detach_load_balancer_from_network",
    "detach_server_from_network",
    "detach_server_iso",
    "detach_volume",
    "disable_load_balancer_public_interface",
    "disable_server_backup",
    "disable_server_rescue",
    "disable_storage_box_snapshot_plan",
    "import_zone_zonefile",
    "poweroff_server",
    "rebuild_server",
    "remove_firewall_from_resources",
    "remove_load_balancer_target",
    "remove_server_from_placement_group",
    "remove_zone_rrset_records",
    "reset_server",
    "reset_server_password",
    "reset_storage_box_password",
    "reset_storage_box_subaccount_password",
    "rollback_storage_box_snapshot",
    "set_firewall_rules",
    "set_zone_rrset_records",
    "shutdown_server",
    "unassign_floating_ip",
    "unassign_primary_ip",
}

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


@dataclass(frozen=True)
class Operation:
    api: str
    method: str
    tag: str
    operation_id: str
    pagination: str
    status: str
    response: str


def read_tsv(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="ascii", newline="") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


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


def load_operations() -> list[Operation]:
    fingerprint_rows = [
        row for row in read_tsv(FINGERPRINTS) if row["deprecated"] == "no"
    ]
    fingerprints = indexed(fingerprint_rows, "operation_id", "fingerprint")
    responses = indexed(read_tsv(RESPONSES), "operation_id", "response")
    bodies = read_bodies(BODIES)
    active = set(fingerprints)
    if len(active) != EXPECTED_OPERATIONS:
        raise ValueError("active operation count changed")
    if set(responses) != active:
        raise ValueError("response bindings do not exactly cover active operations")
    if not bodies <= active:
        raise ValueError("request-body lock contains inactive operations")
    operations = []
    for operation_id in sorted(active):
        source = fingerprints[operation_id]
        response = responses[operation_id]
        if source["api"] != response["api"]:
            raise ValueError(f"API source mismatch for {operation_id}")
        operations.append(
            Operation(
                api=source["api"],
                method=source["method"],
                tag=source["tag"],
                operation_id=operation_id,
                pagination=source["pagination"],
                status=response["status"],
                response=response["shape"],
            )
        )
    return operations


def pascal(value: str) -> str:
    return "".join(part[:1].upper() + part[1:] for part in value.split("_"))


def service(operation: Operation) -> tuple[str, str, str]:
    if operation.api == "hetzner":
        return "StorageService", "StorageEndpointPolicy", "BasicAuthentication"
    if operation.tag in DNS_TAGS:
        return "DnsService", "CloudEndpointPolicy", "BearerAuthentication"
    if operation.tag in SECURITY_TAGS:
        return "SecurityService", "CloudEndpointPolicy", "BearerAuthentication"
    return "CloudService", "CloudEndpointPolicy", "BearerAuthentication"


def query(operation: Operation) -> str:
    if operation.operation_id in REQUIRED_QUERY:
        return "RequiredQuery"
    if (
        operation.pagination == "yes"
        or operation.operation_id in OPTIONAL_QUERY_WITHOUT_PAGINATION
    ):
        return "OptionalQuery"
    return "QueryForbidden"


def permit(operation: Operation) -> str:
    if operation.operation_id in COST_OPERATIONS:
        return "CostPermit"
    if operation.method == "DELETE" or operation.operation_id in DESTRUCTIVE_POST_OPERATIONS:
        return "DestructivePermit"
    if operation.method == "GET":
        return "NoPermit"
    return "MutationPermit"


def row(operation: Operation, bodies: set[str]) -> str:
    service_type, endpoint, authentication = service(operation)
    body = "JsonBody" if operation.operation_id in bodies else "BodyForbidden"
    values = (
        pascal(operation.operation_id),
        f'"{operation.operation_id}"',
        service_type,
        endpoint,
        authentication,
        METHOD_TYPES[operation.method],
        query(operation),
        body,
        STATUS_TYPES[operation.status],
        RESPONSE_TYPES[operation.response],
        "NumberedPagination" if operation.pagination == "yes" else "NoPagination",
        permit(operation),
    )
    return "        (" + ", ".join(values) + "),"


def render() -> str:
    operations = load_operations()
    bodies = read_bodies(BODIES)
    rows = "\n".join(row(operation, bodies) for operation in operations)
    return f'''//! Generated exhaustive active-operation associations.
//!
//! Regenerate with `scripts/generate_operation_associations.py`.

use cloud_sdk::{{ServiceMarker, operation_id}};

use super::policy::{{HetznerOperation, OperationDescriptor, Sealed}};
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
macro_rules! retry {{
    (GetMethod) => {{ ExplicitRetry }};
    (PutMethod) => {{ ExplicitRetry }};
    ($method:ident) => {{ NeverRetry }};
}}

macro_rules! operation_associations {{
    ($(($marker:ident, $id:literal, $service:ident, $endpoint:ident, $authentication:ident,
        $method:ident, $query:ident, $body:ident, $status:ident, $response:ident,
        $pagination:ident, $permit:ident),)+) => {{
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
                    type Retry = retry!($method);
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
                        <$pagination as PaginationAssociation>::POLICY,
                        <$method as MethodAssociation>::RETRY,
                        <$permit as PermitAssociation>::CLASS,
                    );
                }}
            )+
        }}

        /// Descriptors for all active operations in stable operation-ID order.
        pub const ALL_OPERATIONS: &[OperationDescriptor] = &[
            $(<operations::$marker as HetznerOperation>::DESCRIPTOR,)+
        ];
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
