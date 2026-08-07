#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

python3 scripts/check_ovhcloud_header_conformance.py
python3 scripts/test-ovhcloud-header-conformance.py
cargo test --locked -p cloud-sdk --test ovhcloud_header_conformance
cargo test --locked -p cloud-sdk pagination::tests::header_cursor
cargo test --locked -p cloud-sdk schema::tests

echo "OVHcloud cursor and schema header conformance passed."
