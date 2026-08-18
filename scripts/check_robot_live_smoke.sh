#!/usr/bin/env sh
set -eu

python3 scripts/check_robot_live_smoke.py
python3 scripts/test-robot-live-smoke.py
python3 scripts/test-live-smoke-wrapper.py
python3 scripts/test-hetzner-live-smoke-runner.py
cargo test --locked -p cloud-sdk-hetzner --test live_smoke --all-features
