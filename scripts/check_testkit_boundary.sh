#!/usr/bin/env sh
set -eu

default_tree=$(cargo tree -p cloud-sdk-testkit --no-default-features --edges normal,build)
if ! printf '%s\n' "$default_tree" | grep -Fq 'cloud-sdk v0.31.0'; then
    echo "testkit boundary: cloud-sdk v0.31.0 is missing" >&2
    exit 1
fi
if printf '%s\n' "$default_tree" | grep -Eq \
    'cloud-sdk-(hetzner|reqwest|sanitization)|reqwest|hyper|tokio|async-std|smol|rustls|native-tls|openssl|serde|mio|socket'; then
    echo "testkit boundary: forbidden provider, network, TLS, runtime, or parser dependency" >&2
    printf '%s\n' "$default_tree" >&2
    exit 1
fi
if [ "$(printf '%s\n' "$default_tree" | wc -l)" -ne 2 ]; then
    echo "testkit boundary: unexpected default dependency entered graph" >&2
    printf '%s\n' "$default_tree" >&2
    exit 1
fi

cargo check -p cloud-sdk-testkit --no-default-features
cargo test -p cloud-sdk-testkit --all-features
cargo package -p cloud-sdk-testkit --allow-dirty \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"'
