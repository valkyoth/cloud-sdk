#!/usr/bin/env sh
set -eu

. scripts/enforce_bundled_aws_lc.sh

sdk_version=$(awk -F '"' '/^version = "/ { print $2; exit }' Cargo.toml)
if [ -z "$sdk_version" ]; then
    echo "reqwest boundary: workspace version is missing" >&2
    exit 1
fi
default_tree=$(cargo tree -p cloud-sdk-reqwest --no-default-features --edges normal)
default_dependencies=$(printf '%s\n' "$default_tree" | sed '1d')
if ! printf '%s\n' "$default_tree" | grep -Fq "cloud-sdk v${sdk_version}"; then
    echo "reqwest boundary: cloud-sdk v${sdk_version} is missing" >&2
    exit 1
fi
if ! printf '%s\n' "$default_tree" | grep -Fq 'subtle v2.6.1'; then
    echo "reqwest boundary: fixed-time core primitive is missing" >&2
    exit 1
fi
if printf '%s\n' "$default_dependencies" | grep -Eq \
    'reqwest|hyper|tokio|rustls|native-tls|openssl'; then
    echo "reqwest boundary: transport entered the default graph" >&2
    printf '%s\n' "$default_tree" >&2
    exit 1
fi
if [ "$(printf '%s\n' "$default_tree" | wc -l)" -ne 5 ]; then
    echo "reqwest boundary: unexpected default dependency entered graph" >&2
    printf '%s\n' "$default_tree" >&2
    exit 1
fi

std_tree=$(cargo tree -p cloud-sdk-reqwest --no-default-features --features std --edges normal)
std_dependencies=$(printf '%s\n' "$std_tree" | sed '1d')
if printf '%s\n' "$std_dependencies" | grep -Eq \
    'reqwest|hyper|tokio|rustls|native-tls|openssl'; then
    echo "reqwest boundary: transport entered the std-only graph" >&2
    printf '%s\n' "$std_tree" >&2
    exit 1
fi

blocking_tree=$(cargo tree -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls --edges normal)
for dependency in \
    'http v1.5.0' \
    'base64-ng v2.0.1' \
    'http-body-util v0.1.5' \
    'hyper v1.11.0' \
    'hyper-rustls v0.27.9' \
    'hyper-util v0.1.20' \
    'reqwest v0.13.4' \
    'cloud-sdk-sanitization v1.0.0' \
    'sanitization v2.0.3' \
    'rustls v0.23.43'; do
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
    'base64-ng v2.0.1' \
    'http-body-util v0.1.5' \
    'hyper v1.11.0' \
    'hyper-rustls v0.27.9' \
    'hyper-util v0.1.20' \
    'reqwest v0.13.4' \
    'tokio v1.53.1' \
    'cloud-sdk-sanitization v1.0.0' \
    'sanitization v2.0.3' \
    'rustls v0.23.43'; do
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
    'configured_raw_client' \
    'RawHyperClient::new'; do
    if ! grep -Fq "$policy" "$config"; then
        echo "reqwest boundary: required client policy $policy is missing from $config" >&2
        exit 1
    fi
done
done

raw_engine=crates/cloud-sdk-reqwest/src/shared/raw_hyper.rs
for policy in \
    '.https_only()' \
    '.enable_http1()' \
    '.http1_max_headers(super::MAX_UPSTREAM_HTTP1_HEADERS)' \
    '.http1_max_buf_size(super::MAX_UPSTREAM_HTTP1_HEAD_BYTES)' \
    '.pool_max_idle_per_host(0)' \
    '.retry_canceled_requests(false)' \
    'execute_authenticated' \
    'authorization.set_sensitive(true)'; do
    if ! grep -Fq "$policy" "$raw_engine"; then
        echo "reqwest boundary: required raw client policy $policy is missing" >&2
        exit 1
    fi
done
if grep -REn 'reqwest::(blocking::)?Client|Client::builder\(\)' \
    crates/cloud-sdk-reqwest/src/blocking \
    crates/cloud-sdk-reqwest/src/asynchronous; then
    echo "reqwest boundary: legacy high-level reqwest execution re-entered authenticated clients" >&2
    exit 1
fi

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
