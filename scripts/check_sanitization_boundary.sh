#!/usr/bin/env sh
set -eu

boundary_tree=$(cargo tree -p cloud-sdk-sanitization --no-default-features --edges normal)
if ! printf '%s\n' "$boundary_tree" | grep -Fq 'sanitization v1.2.4'; then
    echo "sanitization boundary: admitted sanitization version is missing" >&2
    exit 1
fi
if printf '%s\n' "$boundary_tree" | grep -Eq '(^|[[:space:]])(zeroize|subtle) v'; then
    echo "sanitization boundary: optional interoperability dependency entered graph" >&2
    exit 1
fi
sanitization_tree=$(cargo tree -p sanitization@1.2.4 --edges normal)
if [ "$(printf '%s\n' "$sanitization_tree" | wc -l)" -ne 1 ]; then
    echo "sanitization boundary: default transitive dependency entered graph" >&2
    exit 1
fi

alloc_tree=$(
    cargo tree -p cloud-sdk-sanitization --no-default-features \
        --features alloc --edges normal
)
if ! printf '%s\n' "$alloc_tree" | grep -Fq 'cloud-sdk v'; then
    echo "sanitization boundary: alloc feature did not enable cloud-sdk allocation support" >&2
    exit 1
fi
if printf '%s\n' "$alloc_tree" | grep -Eq '(^|[[:space:]])(zeroize|subtle) v'; then
    echo "sanitization boundary: alloc feature enabled an interoperability dependency" >&2
    exit 1
fi

for package in cloud-sdk cloud-sdk-hetzner; do
    default_tree=$(cargo tree -p "$package" --no-default-features --edges normal)
    if printf '%s\n' "$default_tree" | grep -Eq '(^|[[:space:]])sanitization v'; then
        echo "sanitization boundary: dependency entered $package default graph" >&2
        exit 1
    fi
done

hetzner_serde_tree=$(
    cargo tree -p cloud-sdk-hetzner --no-default-features \
        --features serde --edges normal
)
if ! printf '%s\n' "$hetzner_serde_tree" |
    grep -Fq 'cloud-sdk-sanitization v'; then
    echo "sanitization boundary: Hetzner serde graph is missing owned secret cleanup" >&2
    exit 1
fi
if printf '%s\n' "$hetzner_serde_tree" | grep -Eq '(^|[[:space:]])(zeroize|subtle) v'; then
    echo "sanitization boundary: Hetzner serde graph enabled an interoperability dependency" >&2
    exit 1
fi

cargo test -p cloud-sdk-sanitization --all-features
cargo package -p cloud-sdk-sanitization --allow-dirty --no-verify \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"' >/dev/null
