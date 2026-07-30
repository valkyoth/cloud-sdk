#!/usr/bin/env sh
set -eu

for contract in \
    'pub struct AuthenticationScope' \
    'pub struct AuthenticationScopePolicy' \
    'pub enum ScopeRequirement' \
    'pub struct CredentialGeneration' \
    'pub struct RefreshHandoff' \
    'pub struct AuthenticatedRequest' \
    'pub trait BlockingAuthenticatedTransport' \
    'pub trait AsyncAuthenticatedTransport'; do
    if ! grep -R -Fq "$contract" crates/cloud-sdk/src/authentication*; then
        echo "bearer authentication: missing core contract $contract" >&2
        exit 1
    fi
done

for contract in \
    'pub struct BearerCredential' \
    'pub struct BearerCredentialSnapshot' \
    'pub struct BearerRefreshHandoff' \
    'pub enum TokenRefreshError'; do
    if ! grep -R -Fq "$contract" crates/cloud-sdk-reqwest/src; then
        echo "bearer authentication: missing adapter contract $contract" >&2
        exit 1
    fi
done

if ! grep -Fq 'define_scope!(BearerCredentialScope, "BearerCredentialScope")' \
    crates/cloud-sdk-reqwest/src/shared/scope.rs; then
    echo "bearer authentication: missing adapter contract BearerCredentialScope" >&2
    exit 1
fi

if grep -R -Fq 'BearerCredentialScope::unscoped' \
    crates/cloud-sdk-reqwest crates/cloud-sdk-hetzner/tests; then
    echo "bearer authentication: adapter still permits an unscoped credential" >&2
    exit 1
fi

if ! grep -Fq 'CredentialMismatch' \
    crates/cloud-sdk-reqwest/src/shared/credentials.rs; then
    echo "bearer authentication: refresh handoffs lack store-lineage rejection" >&2
    exit 1
fi

if ! grep -Fq 'validate_test_loopback_scope' \
    crates/cloud-sdk-reqwest/src/shared/authentication.rs; then
    echo "bearer authentication: test loopback bypasses complete scope validation" >&2
    exit 1
fi

if grep -R -Eq \
    'impl (BlockingTransport|AsyncTransport) for (BlockingClient|AsyncClient)' \
    crates/cloud-sdk-reqwest/src; then
    echo "bearer authentication: authenticated client has a policy-free transport bypass" >&2
    exit 1
fi

for implementation in \
    'impl BlockingAuthenticatedTransport for BlockingClient' \
    'impl AsyncAuthenticatedTransport for AsyncClient'; do
    if ! grep -R -Fq "$implementation" crates/cloud-sdk-reqwest/src; then
        echo "bearer authentication: missing mandatory adapter implementation $implementation" >&2
        exit 1
    fi
done

if ! grep -Fq 'validate_bearer_authentication(' \
    crates/cloud-sdk-reqwest/src/blocking/client.rs \
    || ! grep -Fq 'validate_bearer_authentication(' \
        crates/cloud-sdk-reqwest/src/asynchronous/client.rs; then
    echo "bearer authentication: adapter send omits scope validation" >&2
    exit 1
fi

if ! grep -Fq 'Bytes::from_owner(owner)' \
    crates/cloud-sdk-reqwest/src/shared/secret_header.rs \
    || ! grep -Fq 'sanitize_bytes(&mut self.bytes)' \
        crates/cloud-sdk-reqwest/src/shared/secret_header.rs; then
    echo "bearer authentication: authorization header lacks cleanup ownership" >&2
    exit 1
fi

cargo check -p cloud-sdk --no-default-features
cargo test -p cloud-sdk --features std authentication
cargo test -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls authentication
cargo test -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls credentials
cargo test -p cloud-sdk-reqwest --no-default-features \
    --features async-rustls authentication

test -s docs/AUTHENTICATION_POLICY.md
test -s docs/MIGRATION_0.41.0.md
