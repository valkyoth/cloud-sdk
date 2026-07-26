#!/usr/bin/env sh
set -eu

cargo test --locked -p cloud-sdk --all-features transport::endpoint
cargo test --locked -p cloud-sdk-hetzner --all-features endpoint::tests
cargo test --locked -p cloud-sdk-reqwest \
    --features blocking-rustls,async-rustls \
    endpoints_reject_authority_and_normalization_ambiguity
cargo test --locked -p cloud-sdk-testkit --all-features prepared
cargo test --locked -p cloud-sdk --doc --all-features
cargo test --locked -p cloud-sdk-reqwest --doc \
    --features blocking-rustls,async-rustls

core_tree="$(
    cargo tree --locked -p cloud-sdk --all-features \
        --target all --edges normal --prefix none
)"
for forbidden in reqwest tokio hyper hickory-resolver trust-dns-resolver socket2 mio; do
    if printf '%s\n' "$core_tree" | grep -Eq "^${forbidden} v"; then
        echo "endpoint policy: core unexpectedly owns DNS or egress dependency ${forbidden}" >&2
        exit 1
    fi
done

printf '%s\n' "endpoint policy: trust classes, authority corpus, and egress isolation passed."
