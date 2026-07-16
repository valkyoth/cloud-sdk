#!/usr/bin/env sh
set -eu

. scripts/enforce_bundled_aws_lc.sh

default_tree=$(cargo tree -p cloud-sdk-hetzner --no-default-features --edges normal,build)
if printf '%s\n' "$default_tree" | grep -Eq '(^|[[:space:]])serde(_json)? v'; then
    echo "serde boundary: serde entered the default normal graph" >&2
    exit 1
fi

feature_tree=$(cargo tree -p cloud-sdk-hetzner --no-default-features --features serde \
    --edges normal,build -e features)
if ! printf '%s\n' "$feature_tree" | grep -Fq 'serde feature "alloc"'; then
    echo "serde boundary: optional graph is missing serde alloc" >&2
    exit 1
fi
if printf '%s\n' "$feature_tree" | grep -Fq 'serde feature "std"'; then
    echo "serde boundary: serde std must not enter the optional graph" >&2
    exit 1
fi
if ! printf '%s\n' "$feature_tree" | grep -Fq 'serde_json feature "alloc"'; then
    echo "serde boundary: checked decoder is missing serde_json alloc" >&2
    exit 1
fi
if printf '%s\n' "$feature_tree" | grep -Fq 'serde_json feature "std"'; then
    echo "serde boundary: serde_json std must not enter the optional graph" >&2
    exit 1
fi

cargo check -p cloud-sdk-hetzner --no-default-features
cargo check -p cloud-sdk-hetzner --no-default-features --features serde
cargo test -p cloud-sdk-hetzner --all-features serde
