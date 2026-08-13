#!/bin/sh
set -eu

scripts/check_robot_boot.py
scripts/test-robot-boot.py
cargo test --locked -p cloud-sdk-hetzner --all-features robot::boot::
