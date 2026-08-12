#!/bin/sh
set -eu

scripts/check_robot_subnets.py
scripts/test-robot-subnets.py
cargo check --locked --manifest-path fuzz/Cargo.toml --bin robot_subnet_response
