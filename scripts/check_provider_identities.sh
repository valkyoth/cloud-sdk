#!/usr/bin/env sh
set -eu

if grep -R -n -E 'pub enum (Provider|ApiFamily)([[:space:]<{]|$)' crates/cloud-sdk/src; then
    echo "provider identities: closed core provider taxonomy returned" >&2
    exit 1
fi

cargo test -p cloud-sdk --test provider_extensibility --all-features
cargo test -p cloud-sdk --doc --all-features

for identity in \
    HETZNER_PROVIDER_ID \
    CLOUD_SERVICE_ID \
    DNS_SERVICE_ID \
    SECURITY_SERVICE_ID \
    STORAGE_SERVICE_ID; do
    if ! grep -q -E "pub const ${identity}:" \
        crates/cloud-sdk-hetzner/src/identity.rs; then
        echo "provider identities: missing Hetzner identity ${identity}" >&2
        exit 1
    fi
done

echo "Provider and service identities are bounded, extensible, and provider-owned."
