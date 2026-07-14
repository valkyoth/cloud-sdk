#!/usr/bin/env sh
set -eu

fips_tree="$(
    cargo tree -p cloud-sdk-reqwest --no-default-features \
        --features blocking-rustls-fips --edges normal
)"

for dependency in \
    'reqwest v0.13.4' \
    'rustls v0.23.42' \
    'rustls-platform-verifier v0.7.0' \
    'aws-lc-rs v1.17.1' \
    'aws-lc-sys v0.42.0' \
    'aws-lc-fips-sys v0.13.15'; do
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

if ! grep -Fq 'rustls::crypto::default_fips_provider()' \
    crates/cloud-sdk-reqwest/src/blocking/config.rs \
    || ! grep -Fq 'validate_fips_provider(provider.as_ref())?' \
        crates/cloud-sdk-reqwest/src/blocking/config.rs \
    || ! grep -Fq 'validate_fips_config(&config)?' \
        crates/cloud-sdk-reqwest/src/blocking/config.rs; then
    echo "reqwest FIPS boundary: explicit runtime verification is missing" >&2
    exit 1
fi

AWS_LC_FIPS_SYS_USE_SYSTEM=0 cargo test -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls-fips
AWS_LC_FIPS_SYS_USE_SYSTEM=0 cargo check -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls,blocking-rustls-fips
