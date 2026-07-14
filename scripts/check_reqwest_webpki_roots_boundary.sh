#!/usr/bin/env sh
set -eu

. scripts/enforce_bundled_aws_lc.sh

roots_tree="$(
    cargo tree -p cloud-sdk-reqwest --no-default-features \
        --features blocking-rustls-webpki-roots --edges normal
)"

for dependency in \
    'reqwest v0.13.4' \
    'rustls v0.23.42' \
    'aws-lc-rs v1.17.1' \
    'aws-lc-sys v0.42.0' \
    'webpki-roots v1.0.8'; do
    if ! printf '%s\n' "$roots_tree" | grep -Fq "$dependency"; then
        echo "reqwest WebPKI-roots boundary: required dependency $dependency is missing" >&2
        exit 1
    fi
done

if printf '%s\n' "$roots_tree" | grep -Eq \
    'native-tls|openssl-sys|aws-lc-fips-sys|ring v|flate2|brotli v|zstd v|async-compression'; then
    echo "reqwest WebPKI-roots boundary: forbidden TLS, FIPS, or decompression dependency" >&2
    printf '%s\n' "$roots_tree" >&2
    exit 1
fi

feature_tree="$(
    cargo tree -p cloud-sdk-reqwest --no-default-features \
        --features blocking-rustls-webpki-roots --edges features,no-dev -i reqwest
)"
for feature in 'reqwest feature "blocking"' 'reqwest feature "rustls-no-provider"'; do
    if ! printf '%s\n' "$feature_tree" | grep -Fq "$feature"; then
        echo "reqwest WebPKI-roots boundary: required $feature is missing" >&2
        exit 1
    fi
done
if printf '%s\n' "$feature_tree" | grep -Eq \
    'reqwest feature "(default|native-tls|gzip|brotli|zstd|deflate|cookies|hickory-dns|http2|json|multipart|socks)"'; then
    echo "reqwest WebPKI-roots boundary: unreviewed reqwest feature entered graph" >&2
    printf '%s\n' "$feature_tree" >&2
    exit 1
fi

cargo test -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls-webpki-roots
cargo check -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls,blocking-rustls-webpki-roots
cargo check -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls-webpki-roots,blocking-rustls-fips
