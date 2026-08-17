#!/usr/bin/env sh
set -eu

cargo test -p cloud-sdk-hetzner --all-features --lib \
    client::robot::coverage_tests
cargo test -p cloud-sdk-hetzner --all-features --test robot_client
