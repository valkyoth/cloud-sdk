#!/usr/bin/env sh
set -eu

if grep -R -n 'method_metadata' \
    crates/cloud-sdk-hetzner/src/prepared \
    tools/prepared-coverage-check/locks/endpoints.rs; then
    echo "HTTP methods: provider safety metadata must not be method-derived" >&2
    exit 1
fi

cargo test -p cloud-sdk method
cargo test -p cloud-sdk-testkit mock_transport_matches_extension_methods_without_aliasing
cargo test -p cloud-sdk-hetzner prepared::operation::tests
cargo test -p cloud-sdk-reqwest --features blocking-rustls,async-rustls \
    complete_method_domain_exactly

echo "HTTP method domain and provider-owned operation metadata passed."
