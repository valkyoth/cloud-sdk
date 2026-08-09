#!/usr/bin/env sh
set -eu

cargo test -p cloud-sdk --all-features client::profile
cargo test -p cloud-sdk-hetzner --all-features client::
cargo test -p cloud-sdk-hetzner --all-features --test client_foundation
cargo test -p cloud-sdk-hetzner --all-features --doc
cargo check -p cloud-sdk --no-default-features
cargo check -p cloud-sdk-hetzner --no-default-features

echo "Hetzner client construction, trust, storage, and read-only execution checks passed."
