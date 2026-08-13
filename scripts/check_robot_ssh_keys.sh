#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
python3 "$ROOT/scripts/check_robot_ssh_keys.py" "$@"
cargo test --locked -p cloud-sdk-hetzner --all-features robot::ssh_keys
cargo check --locked --manifest-path "$ROOT/fuzz/Cargo.toml" --bin robot_ssh_key_response
