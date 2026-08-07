#!/usr/bin/env sh
set -eu

scripts/check_hetzner_vertical_slices.py
cargo test --locked -p cloud-sdk-hetzner --all-features --lib vertical_tests
cargo test --locked -p cloud-sdk-hetzner --all-features --lib \
    neutral_freeze_slices_all_prepare_through_exact_associations
cargo test --locked -p cloud-sdk-hetzner --all-features --test vertical_execution
cargo test --locked -p cloud-sdk-testkit --all-features \
    exact_success_status_and_delivery_classification_support_permit_fixtures
