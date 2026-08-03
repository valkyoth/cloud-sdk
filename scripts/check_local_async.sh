#!/usr/bin/env sh
set -eu

cargo test --locked -p cloud-sdk --all-features local_async
cargo test --locked -p cloud-sdk-testkit --all-features local_async
cargo test --locked -p cloud-sdk --doc --all-features
cargo test --locked -p cloud-sdk-testkit --doc --all-features

echo "Local async contracts, cancellation, conformance, and doctests passed."
