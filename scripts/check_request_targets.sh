#!/usr/bin/env sh
set -eu

cargo test --locked -p cloud-sdk --all-features transport::request_target
cargo test --locked -p cloud-sdk-hetzner --all-features \
    prepared::tests::prepares_global_actions_and_catalog_gets
cargo test --locked -p cloud-sdk-reqwest \
    --features blocking-rustls,async-rustls \
    canonical_request_targets_preserve_exact_wire_bytes
cargo test --locked -p cloud-sdk-testkit --all-features \
    mock_transport_distinguishes_query_presence_and_dialect
cargo check --locked --manifest-path fuzz/Cargo.toml --bin request_targets

core_tree="$(
    cargo tree --locked -p cloud-sdk --all-features \
        --target all --edges normal --prefix none
)"
for forbidden in url percent-encoding form_urlencoded reqwest; do
    if printf '%s\n' "$core_tree" | grep -Eq "^${forbidden} v"; then
        echo "request targets: core unexpectedly depends on ${forbidden}" >&2
        exit 1
    fi
done

printf '%s\n' "Canonical path/query vectors and cross-adapter corpus passed."
