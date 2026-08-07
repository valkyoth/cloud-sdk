#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

python3 scripts/check_ovhcloud_task_conformance.py
python3 scripts/test-ovhcloud-task-conformance.py
cargo test --locked -p cloud-sdk --test ovhcloud_task_conformance
cargo test --locked -p cloud-sdk async_resource

echo "OVHcloud task and event-model conformance passed."
