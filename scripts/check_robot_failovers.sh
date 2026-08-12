#!/bin/sh
set -eu

scripts/check_robot_failovers.py
scripts/test-robot-failovers.py
cargo test --locked -p cloud-sdk-hetzner --all-features robot::failover::
cargo check --locked --manifest-path fuzz/Cargo.toml --bin robot_failover_response
