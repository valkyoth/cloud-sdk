#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
python3 "$ROOT/scripts/check_robot_vswitches.py" "$@"
cargo test --locked -p cloud-sdk-hetzner --all-features robot::vswitch
cargo check --locked --manifest-path "$ROOT/fuzz/Cargo.toml" --bin robot_vswitch_response
