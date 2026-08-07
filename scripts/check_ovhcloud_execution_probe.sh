#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."

python3 scripts/check_ovhcloud_probe.py
python3 scripts/test-ovhcloud-probe.py
cargo test --locked -p ovhcloud-v2-probe
cargo test --locked -p ovhcloud-v2-probe --all-features

echo "OVHcloud end-to-end execution probe passed; live smoke remained ignored."
