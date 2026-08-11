#!/usr/bin/env sh
set -eu

scripts/check_robot_cancellations.py
scripts/test-robot-cancellations.py
cargo check --locked -p cloud-sdk-hetzner --no-default-features --features alloc
cargo check --locked -p cloud-sdk-hetzner --no-default-features --features serde
cargo test --locked -p cloud-sdk-hetzner --no-default-features --features serde robot::cancellation
cargo check --locked --manifest-path fuzz/Cargo.toml --bin robot_cancellation_response

echo "Robot cancellation request, response, and source contracts passed."
