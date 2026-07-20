#!/usr/bin/env sh
set -eu

. scripts/enforce_bundled_aws_lc.sh

default_tree=$(cargo tree -p cloud-sdk-reqwest --no-default-features --edges normal)
default_dependencies=$(printf '%s\n' "$default_tree" | sed '1d')
if ! printf '%s\n' "$default_tree" | grep -Fq 'cloud-sdk v0.31.0'; then
    echo "reqwest boundary: cloud-sdk v0.31.0 is missing" >&2
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
    'cloud-sdk-sanitization v0.14.0' \
    'sanitization v1.2.5' \
    'rustls v0.23.42'; do
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

async_tree=$(cargo tree -p cloud-sdk-reqwest --no-default-features \
    --features async-rustls --edges normal)
for dependency in \
    'bytes v1.12.1' \
    'reqwest v0.13.4' \
    'tokio v1.53.0' \
    'cloud-sdk-sanitization v0.14.0' \
    'sanitization v1.2.5' \
    'rustls v0.23.42'; do
    if ! printf '%s\n' "$async_tree" | grep -Fq "$dependency"; then
        echo "reqwest boundary: admitted async dependency $dependency is missing" >&2
        exit 1
    fi
done
if printf '%s\n' "$async_tree" | grep -Eq \
    'native-tls|openssl-sys|flate2|brotli v|zstd v|async-compression'; then
    echo "reqwest boundary: native TLS or response decompression entered async graph" >&2
    printf '%s\n' "$async_tree" >&2
    exit 1
fi

legacy_windows_tree=$(cargo tree -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls --target all --edges normal \
    -i windows-sys@0.52.0 2>/dev/null || true)
if printf '%s\n' "$legacy_windows_tree" | grep -Fq 'windows-sys v0.52.0'; then
    echo "reqwest boundary: legacy windows-sys 0.52 re-entered the active graph" >&2
    exit 1
fi

feature_tree=$(cargo tree -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls --edges features,no-dev -i reqwest)
for feature in 'reqwest feature "blocking"' 'reqwest feature "rustls"'; do
    if ! printf '%s\n' "$feature_tree" | grep -Fq "$feature"; then
        echo "reqwest boundary: required $feature is missing" >&2
        exit 1
    fi
done
if printf '%s\n' "$feature_tree" | grep -Eq \
    'reqwest feature "(default|native-tls|gzip|brotli|zstd|deflate|cookies|hickory-dns|http2|json|multipart|socks)"'; then
    echo "reqwest boundary: unreviewed reqwest feature entered graph" >&2
    printf '%s\n' "$feature_tree" >&2
    exit 1
fi

async_feature_tree=$(cargo tree -p cloud-sdk-reqwest --no-default-features \
    --features async-rustls --edges features,no-dev -i reqwest)
if ! printf '%s\n' "$async_feature_tree" | grep -Fq 'reqwest feature "rustls"'; then
    echo "reqwest boundary: async graph is missing reqwest rustls" >&2
    exit 1
fi
if printf '%s\n' "$async_feature_tree" | grep -Eq \
    'reqwest feature "(blocking|default|native-tls|gzip|brotli|zstd|deflate|cookies|hickory-dns|http2|json|multipart|socks)"'; then
    echo "reqwest boundary: unreviewed reqwest feature entered async graph" >&2
    printf '%s\n' "$async_feature_tree" >&2
    exit 1
fi

adversarial_tree=$(cargo tree --manifest-path tests/reqwest-feature-unification/Cargo.toml \
    --locked --edges features -i reqwest)
for feature in 'reqwest feature "hickory-dns"' 'reqwest feature "http2"'; do
    if ! printf '%s\n' "$adversarial_tree" | grep -Fq "$feature"; then
        echo "reqwest boundary: adversarial test graph is missing $feature" >&2
        exit 1
    fi
done

for package in cloud-sdk cloud-sdk-hetzner; do
    package_tree=$(cargo tree -p "$package" --no-default-features --edges normal)
    if printf '%s\n' "$package_tree" | grep -Eq 'reqwest|hyper|tokio|rustls'; then
        echo "reqwest boundary: transport entered $package default graph" >&2
        exit 1
    fi
done

for source in \
    crates/cloud-sdk-reqwest/src/blocking/client.rs \
    crates/cloud-sdk-reqwest/src/asynchronous/client.rs \
    crates/cloud-sdk-reqwest/src/shared/credentials.rs; do
    if grep -En \
        'tokio::(spawn|task|time::sleep)|std::thread|thread::spawn|Semaphore|Runtime::' \
        "$source"; then
        echo "reqwest boundary: background execution entered $source" >&2
        exit 1
    fi
done

if find crates -name Cargo.toml -exec grep -HnE '(^|[[:space:]])zeroize([[:space:]]|=)' {} +; then
    echo "reqwest boundary: first-party manifests must use cloud-sdk-sanitization" >&2
    exit 1
fi

for config in \
    crates/cloud-sdk-reqwest/src/blocking/config.rs \
    crates/cloud-sdk-reqwest/src/asynchronous/config.rs; do
for policy in \
    '.build_inner(true)' \
    '.tls_backend_rustls()' \
    '.http1_only()' \
    '.no_hickory_dns()' \
    '.min_tls_version(Version::TLS_1_2)' \
    '.redirect(Policy::none())' \
    '.retry(reqwest::retry::never())' \
    '.referer(false)' \
    '.no_proxy()' \
    '.no_gzip()' \
    '.no_brotli()' \
    '.no_zstd()' \
    '.no_deflate()'; do
    if ! grep -Fq "$policy" "$config"; then
        echo "reqwest boundary: required client policy $policy is missing from $config" >&2
        exit 1
    fi
done
done

cargo check -p cloud-sdk-reqwest --no-default-features
cargo check -p cloud-sdk-reqwest --no-default-features --features std
cargo test -p cloud-sdk-reqwest --no-default-features --features blocking-rustls
cargo test -p cloud-sdk-reqwest --no-default-features --features async-rustls
cargo test -p cloud-sdk-reqwest --all-features
cargo fmt --manifest-path tests/reqwest-feature-unification/Cargo.toml -- --check
cargo clippy --manifest-path tests/reqwest-feature-unification/Cargo.toml \
    --locked --all-targets -- -D warnings
cargo test --manifest-path tests/reqwest-feature-unification/Cargo.toml --locked
cargo package -p cloud-sdk-reqwest --allow-dirty --all-features \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"' \
    --config 'patch.crates-io.cloud-sdk-sanitization.path="crates/cloud-sdk-sanitization"'
