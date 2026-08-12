#!/bin/sh
set -eu

scripts/check_robot_resets.py
scripts/test-robot-resets.py
cargo test --locked -p cloud-sdk-hetzner --all-features robot::reset::
cargo check --locked --manifest-path fuzz/Cargo.toml --bin robot_reset_response
