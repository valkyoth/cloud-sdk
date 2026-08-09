#!/usr/bin/env sh
set -eu

scripts/generate_dns_client_methods.py --check
scripts/test-dns-client-methods.py
cargo test -p cloud-sdk-hetzner --all-features --test dns_client
cargo test -p cloud-sdk-hetzner --all-features --test dns_client_unpolled_cleanup
cargo check -p cloud-sdk-hetzner --no-default-features

echo "All 24 DNS operations have source-locked client methods and executor evidence."
