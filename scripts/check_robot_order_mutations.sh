#!/usr/bin/env sh
set -eu

python3 scripts/check_robot_order_mutations.py
cargo test -p cloud-sdk-hetzner --features serde robot::ordering::mutation
