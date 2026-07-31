#!/usr/bin/env python3
"""Require all active Hetzner operations to use the authenticated raw wire path."""

from __future__ import annotations

import argparse
from pathlib import Path

from check_api_matrix_coverage import parse_operations

ROOT = Path(__file__).resolve().parents[1]
EXPECTED_ACTIVE = 208

FILES = {
    "core_prepared": "crates/cloud-sdk/src/operation/prepared.rs",
    "core_authenticated": "crates/cloud-sdk/src/authentication/transport.rs",
    "core_raw": "crates/cloud-sdk/src/transport/raw.rs",
    "provider_operation": "crates/cloud-sdk-hetzner/src/prepared/operation.rs",
    "provider_endpoints": "crates/cloud-sdk-hetzner/src/prepared/endpoints.rs",
    "provider_policy": "crates/cloud-sdk-hetzner/src/prepared/wire_policy.rs",
    "blocking_client": "crates/cloud-sdk-reqwest/src/blocking/client.rs",
    "async_client": "crates/cloud-sdk-reqwest/src/asynchronous/client.rs",
    "blocking_basic": "crates/cloud-sdk-reqwest/src/blocking/basic_client.rs",
    "async_basic": "crates/cloud-sdk-reqwest/src/asynchronous/basic_client.rs",
    "raw_hyper": "crates/cloud-sdk-reqwest/src/shared/raw_hyper.rs",
    "live_smoke": "crates/cloud-sdk-hetzner/tests/live_smoke/catalog.rs",
}

REQUIRED = {
    "core_prepared": [
        "authentication_policy: AuthenticationScopePolicy",
        "raw_response_policy: RawResponsePolicy",
        "T: BlockingAuthenticatedTransport + BoundTransport",
        "T: AsyncAuthenticatedTransport + BoundTransport",
        ".send_authenticated(self.authenticated_request(), response.writer())",
    ],
    "core_authenticated": [
        "response_policy: RawResponsePolicy",
        "pub const fn response_policy",
    ],
    "core_raw": [
        "admitted_headers: [Option<HeaderName",
        "pub const fn max_body_bytes",
    ],
    "provider_operation": [
        "fn endpoint_group(self) -> EndpointGroup;",
        "provider_service(endpoint.endpoint_group())",
        "authentication_policy(service, endpoint.api_base_url())",
        "raw_response_policy(profile)",
        "PreparedRequest::new(",
    ],
    "provider_endpoints": [
        "fn endpoint_group(self) -> crate::EndpointGroup",
        "<$type>::endpoint_group(self)",
    ],
    "provider_policy": [
        "ApiSurface::Cloud => ProviderService::from_marker::<CloudService>",
        "ApiSurface::Dns => ProviderService::from_marker::<DnsService>",
        "ApiSurface::Security => ProviderService::from_marker::<SecurityService>",
        "ApiSurface::Storage => ProviderService::from_marker::<StorageService>",
        "ScopeRequirement::Required(service.provider_id())",
        "ScopeRequirement::Required(service.service_id())",
        "ScopeRequirement::Required(endpoint)",
        "RawResponsePolicy::new(",
    ],
    "blocking_client": [
        "client: RawBlockingClient",
        ".execute_authenticated(",
        "authenticated.response_policy()",
    ],
    "async_client": [
        "client: RawAsyncClient",
        ".execute_authenticated(",
        "authenticated.response_policy()",
    ],
    "blocking_basic": ["client: RawBlockingClient", ".execute_authenticated("],
    "async_basic": ["client: RawAsyncClient", ".execute_authenticated("],
    "raw_hyper": [
        "pub(crate) async fn execute_authenticated",
        "headers.insert(AUTHORIZATION, authorization)",
    ],
    "live_smoke": [
        ".prepare(storage)",
        "prepared.authenticated_request()",
    ],
}

FORBIDDEN = {
    "core_prepared": ["T: BlockingTransport", "T: AsyncTransport", ".send(self.request"],
    "provider_operation": ["ApiBaseUrl::CloudV1 => ProviderService"],
    "blocking_client": [
        "reqwest::blocking::Client",
        "ResponseAttempt",
        ".request(method",
        "outbound.send()",
    ],
    "async_client": [
        "use reqwest::{Body, Client}",
        "ResponseAttempt",
        "outbound.send().await",
    ],
    "blocking_basic": ["reqwest::blocking::Client", "super::client::execute"],
    "async_basic": ["use reqwest::Client", "super::client::execute"],
    "live_smoke": ["AuthenticationScopePolicy", "AuthenticatedRequest::new"],
}


def read_sources(root: Path) -> dict[str, str]:
    """Read every source bound to the migration evidence."""
    sources = {}
    for name, relative in FILES.items():
        path = root / relative
        sources[name] = path.read_text(encoding="utf-8")
    return sources


def validate_sources(sources: dict[str, str]) -> None:
    """Reject missing controls and every reviewed compatibility fallback."""
    for name, needles in REQUIRED.items():
        for needle in needles:
            if needle not in sources[name]:
                raise ValueError(f"{name} is missing required wire control: {needle}")
    for name, needles in FORBIDDEN.items():
        for needle in needles:
            if needle in sources[name]:
                raise ValueError(f"{name} contains compatibility fallback: {needle}")
    if sources["provider_operation"].count("PreparedRequest::new(") != 1:
        raise ValueError("provider preparation must have exactly one wire assembly point")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--expected-active", type=int, default=EXPECTED_ACTIVE)
    args = parser.parse_args()
    root = args.root.resolve()
    try:
        operations = parse_operations(root / "docs" / "API_MATRIX.md")
        active = [operation for operation in operations if operation.deprecated == "no"]
        if len(active) != args.expected_active:
            raise ValueError("active operation count changed unexpectedly")
        validate_sources(read_sources(root))
    except (OSError, UnicodeError, ValueError) as error:
        raise SystemExit(f"Hetzner wire migration: {error}") from error
    print(
        f"Hetzner wire migration: {len(active)} active operations use one "
        "authenticated raw path with no compatibility fallback."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
