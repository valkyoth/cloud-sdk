#!/usr/bin/env sh
set -eu

scripts/generate_security_client_methods.py --check
scripts/test-security-client-methods.py
cargo test -p cloud-sdk-hetzner --all-features --test security_client
cargo test -p cloud-sdk-hetzner --all-features --test security_client_unpolled_cleanup
cargo check -p cloud-sdk-hetzner --no-default-features

echo "All 14 Security operations have source-locked client methods and executor evidence."
