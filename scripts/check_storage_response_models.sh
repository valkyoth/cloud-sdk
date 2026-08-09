#!/usr/bin/env sh
set -eu

python3 scripts/test-generate-cloud-model-schema.py
cargo test -p cloud-sdk-hetzner --all-features \
    serde::checked_storage_response_tests
cargo test -p cloud-sdk-hetzner --all-features \
    serde::models::storage_box
cargo test -p cloud-sdk-hetzner --all-features --test vertical_execution
cargo test -p cloud-sdk-hetzner --all-features --test live_smoke
cargo test --manifest-path fuzz/Cargo.toml --test cloud_special_response_seeds

echo "Hetzner Storage Box response models passed source, identity, secret, live-harness, vertical, and fuzz-seed checks."
