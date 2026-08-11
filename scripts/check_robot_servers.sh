#!/usr/bin/env sh
set -eu

scripts/check_robot_server_contract.py
scripts/test-robot-server-contract.py
cargo check --locked -p cloud-sdk-hetzner --no-default-features
cargo check --locked -p cloud-sdk-hetzner --no-default-features --features serde
cargo test --locked -p cloud-sdk-hetzner --no-default-features --features serde robot::server

echo "Robot server request, model, decoder, and source contracts passed."
