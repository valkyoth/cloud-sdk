#!/usr/bin/env sh
set -eu

scripts/generate_cloud_client_methods.py --check
scripts/test-cloud-client-methods.py
cargo test -p cloud-sdk-hetzner --all-features --test client_foundation
cargo test -p cloud-sdk-hetzner --all-features --test vertical_execution \
    action_and_no_content_slices_cross_permit_and_executor_paths
cargo test -p cloud-sdk-hetzner --all-features \
    --test cloud_client_unpolled_cleanup
cargo check -p cloud-sdk-hetzner --no-default-features

echo "All 139 Cloud operations have source-locked client methods and executor evidence."
