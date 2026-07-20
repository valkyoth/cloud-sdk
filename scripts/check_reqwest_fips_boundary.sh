#!/usr/bin/env sh
set -eu

. scripts/enforce_bundled_aws_lc.sh

scripts/check_fips_manifest.py

fips_tree="$(
    cargo tree -p cloud-sdk-reqwest --no-default-features \
        --features blocking-rustls-fips --edges normal
)"

for dependency in \
    'reqwest v0.13.4' \
    'rustls v0.23.42' \
    'rustls-platform-verifier v0.7.0' \
    'aws-lc-rs v1.17.3' \
    'aws-lc-sys v0.43.0' \
    'aws-lc-fips-sys v0.13.16'; do
    if ! printf '%s\n' "$fips_tree" | grep -Fq "$dependency"; then
        echo "reqwest FIPS boundary: required dependency $dependency is missing" >&2
        exit 1
    fi
done

if printf '%s\n' "$fips_tree" | grep -Eq \
    'native-tls|openssl-sys|ring v|flate2|brotli v|zstd v|async-compression'; then
    echo "reqwest FIPS boundary: forbidden TLS, crypto, or decompression dependency" >&2
    printf '%s\n' "$fips_tree" >&2
    exit 1
fi

cargo test -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls-fips
cargo check -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls,blocking-rustls-fips
