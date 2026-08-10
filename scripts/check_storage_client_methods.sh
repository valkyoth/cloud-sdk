#!/usr/bin/env sh
set -eu

scripts/generate_storage_client_methods.py --check
scripts/test-storage-client-methods.py
cargo test -p cloud-sdk-hetzner --all-features --test storage_client
cargo test -p cloud-sdk-hetzner --all-features --test storage_client_unpolled_cleanup
cargo check -p cloud-sdk-hetzner --all-features --example storage_client
echo "All 31 Storage operations have source-locked client methods and executor evidence."
