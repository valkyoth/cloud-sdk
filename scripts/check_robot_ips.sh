#!/usr/bin/env sh
set -eu

scripts/check_robot_ips.py
cargo test --locked -p cloud-sdk-hetzner --all-features robot::ip
cargo check --locked --manifest-path fuzz/Cargo.toml --bin robot_ip_response
