#!/usr/bin/env sh
set -eu

default_tree=$(cargo tree -p cloud-sdk-reqwest --no-default-features --edges normal)
default_dependencies=$(printf '%s\n' "$default_tree" | sed '1d')
if ! printf '%s\n' "$default_tree" | grep -Fq 'cloud-sdk v0.16.0'; then
    echo "reqwest boundary: cloud-sdk v0.16.0 is missing" >&2
    exit 1
fi
if printf '%s\n' "$default_dependencies" | grep -Eq \
    'reqwest|hyper|tokio|rustls|native-tls|openssl|cloud-sdk-sanitization|sanitization'; then
    echo "reqwest boundary: transport or sanitization entered the default graph" >&2
    printf '%s\n' "$default_tree" >&2
    exit 1
fi
if [ "$(printf '%s\n' "$default_tree" | wc -l)" -ne 2 ]; then
    echo "reqwest boundary: unexpected default dependency entered graph" >&2
    printf '%s\n' "$default_tree" >&2
    exit 1
fi

std_tree=$(cargo tree -p cloud-sdk-reqwest --no-default-features --features std --edges normal)
std_dependencies=$(printf '%s\n' "$std_tree" | sed '1d')
if printf '%s\n' "$std_dependencies" | grep -Eq \
    'reqwest|hyper|tokio|rustls|native-tls|openssl|cloud-sdk-sanitization|sanitization'; then
    echo "reqwest boundary: transport or sanitization entered the std-only graph" >&2
    printf '%s\n' "$std_tree" >&2
    exit 1
fi

blocking_tree=$(cargo tree -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls --edges normal)
for dependency in \
    'reqwest v0.13.4' \
    'cloud-sdk-sanitization v0.13.2' \
    'sanitization v1.2.4' \
    'rustls v0.23.41'; do
    if ! printf '%s\n' "$blocking_tree" | grep -Fq "$dependency"; then
        echo "reqwest boundary: admitted dependency $dependency is missing" >&2
        exit 1
    fi
done
if printf '%s\n' "$blocking_tree" | grep -Eq \
    'native-tls|openssl-sys|flate2|brotli v|zstd v|async-compression'; then
    echo "reqwest boundary: native TLS or response decompression entered graph" >&2
    printf '%s\n' "$blocking_tree" >&2
    exit 1
fi

feature_tree=$(cargo tree -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls --edges features -i reqwest)
for feature in 'reqwest feature "blocking"' 'reqwest feature "rustls"'; do
    if ! printf '%s\n' "$feature_tree" | grep -Fq "$feature"; then
        echo "reqwest boundary: required $feature is missing" >&2
        exit 1
    fi
done
if printf '%s\n' "$feature_tree" | grep -Eq \
    'reqwest feature "(default|native-tls|gzip|brotli|zstd|deflate|cookies|json|multipart|socks)"'; then
    echo "reqwest boundary: unreviewed reqwest feature entered graph" >&2
    printf '%s\n' "$feature_tree" >&2
    exit 1
fi

for package in cloud-sdk cloud-sdk-hetzner; do
    package_tree=$(cargo tree -p "$package" --no-default-features --edges normal)
    if printf '%s\n' "$package_tree" | grep -Eq 'reqwest|hyper|tokio|rustls'; then
        echo "reqwest boundary: transport entered $package default graph" >&2
        exit 1
    fi
done

if find crates -name Cargo.toml -exec grep -HnE '(^|[[:space:]])zeroize([[:space:]]|=)' {} +; then
    echo "reqwest boundary: first-party manifests must use cloud-sdk-sanitization" >&2
    exit 1
fi

for policy in \
    '.build_inner(true)' \
    '.min_tls_version(Version::TLS_1_2)' \
    '.redirect(Policy::none())' \
    '.retry(reqwest::retry::never())' \
    '.referer(false)' \
    '.no_proxy()' \
    '.no_gzip()' \
    '.no_brotli()' \
    '.no_zstd()' \
    '.no_deflate()'; do
    if ! grep -Fq "$policy" crates/cloud-sdk-reqwest/src/blocking/config.rs; then
        echo "reqwest boundary: required client policy $policy is missing" >&2
        exit 1
    fi
done

cargo check -p cloud-sdk-reqwest --no-default-features
cargo check -p cloud-sdk-reqwest --no-default-features --features std
cargo test -p cloud-sdk-reqwest --all-features
cargo package -p cloud-sdk-reqwest --allow-dirty --features blocking-rustls \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"' \
    --config 'patch.crates-io.cloud-sdk-sanitization.path="crates/cloud-sdk-sanitization"'
