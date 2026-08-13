#!/bin/sh
set -eu

scripts/check_robot_rdns.py
scripts/test-robot-rdns.py
cargo test --locked -p cloud-sdk-hetzner --all-features robot::rdns::
cargo check --locked --manifest-path fuzz/Cargo.toml --bin robot_rdns_response
