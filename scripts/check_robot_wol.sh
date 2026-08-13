#!/bin/sh
set -eu

scripts/check_robot_wol.py
scripts/test-robot-wol.py
cargo test --locked -p cloud-sdk-hetzner --all-features robot::wol::
