#!/usr/bin/env sh
set -eu

cargo test -p cloud-sdk --all-features client::profile
cargo test -p cloud-sdk-hetzner --all-features client::
cargo test -p cloud-sdk-hetzner --all-features --test client_foundation
cargo check -p cloud-sdk --no-default-features
cargo check -p cloud-sdk-hetzner --no-default-features

grep -Fq 'CloudService, cloud, cloud_with_custom_endpoint' \
    crates/cloud-sdk-hetzner/src/client/construction.rs
grep -Fq 'DnsService, dns, dns_with_custom_endpoint' \
    crates/cloud-sdk-hetzner/src/client/construction.rs
grep -Fq 'SecurityService,' \
    crates/cloud-sdk-hetzner/src/client/construction.rs
grep -Fq 'StorageService,' \
    crates/cloud-sdk-hetzner/src/client/construction.rs
grep -Fq 'HetznerClient<T, S, OfficialEndpointTrust>' \
    crates/cloud-sdk-hetzner/src/client/execution.rs
if grep -Fq 'HetznerClient<T, S, CustomEndpointTrust>' \
    crates/cloud-sdk-hetzner/src/client/execution.rs; then
    echo "Hetzner client foundation: custom trust unexpectedly exposes execution" >&2
    exit 1
fi
grep -Fq 'O: ReadOnlyOperation' \
    crates/cloud-sdk-hetzner/src/association/client.rs
grep -Fq 'CustomEndpointAcknowledgement' docs/HETZNER_CLIENT.md
grep -Fq 'must never come from a tenant' docs/HETZNER_CLIENT.md

echo "Hetzner client construction, trust, storage, and read-only execution checks passed."
