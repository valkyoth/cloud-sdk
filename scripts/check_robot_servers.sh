#!/usr/bin/env sh
set -eu

scripts/check_robot_server_contract.py
scripts/test-robot-server-contract.py
cargo check --locked -p cloud-sdk-hetzner --no-default-features
cargo check --locked -p cloud-sdk-hetzner --no-default-features --features serde
cargo test --locked -p cloud-sdk-hetzner --no-default-features --features serde robot::server
cargo check --locked --manifest-path fuzz/Cargo.toml --bin robot_server_response

echo "Robot server request, protected model, decoder, fuzz, and source contracts passed."
