#!/usr/bin/env sh
set -eu

cargo check -p cloud-sdk --no-default-features
cargo test -p cloud-sdk --no-default-features authentication::attempt
cargo test -p cloud-sdk --no-default-features --test credential_attempt_concurrency
cargo check -p cloud-sdk-hetzner --no-default-features
cargo test -p cloud-sdk-hetzner --no-default-features --features alloc robot::credentials
cargo test -p cloud-sdk-hetzner --no-default-features --features alloc --doc robot::credentials

identity='crates/cloud-sdk-hetzner/src/identity.rs'
endpoint='crates/cloud-sdk-hetzner/src/endpoint.rs'
credentials='crates/cloud-sdk-hetzner/src/robot/credentials.rs'
attempt='crates/cloud-sdk/src/authentication/attempt.rs'

for contract in \
    'pub const ROBOT_SERVICE_ID:' \
    'pub enum RobotService'; do
    grep -Fq "$contract" "$identity" || {
        echo "Robot credentials: missing service identity $contract" >&2
        exit 1
    }
done
for contract in \
    'pub const ROBOT_API_BASE_URL: &str = "https://robot-ws.your-server.de"' \
    'pub fn official_robot_endpoint_policy()' \
    'pub fn verify_official_robot_endpoint('; do
    grep -Fq "$contract" "$endpoint" || {
        echo "Robot credentials: missing endpoint contract $contract" >&2
        exit 1
    }
done
for contract in \
    'pub struct RobotCredentials' \
    'pub struct RobotCredentialScope' \
    'pub struct RobotCredentialAttempt' \
    'pub fn from_mut_bytes(' \
    'pub fn from_secret_buffers(' \
    'pub fn rotate_from_mut_bytes(' \
    'pub fn rotate_from_secret_buffers(' \
    'pub fn reject_attempt(' \
    'pub fn reconfirm(' \
    'pub fn try_with_attempt<T>'; do
    grep -Fq "$contract" "$credentials" || {
        echo "Robot credentials: missing protected lifecycle contract $contract" >&2
        exit 1
    }
done
for contract in \
    'pub struct SharedCredentialAttemptState' \
    'pub struct CredentialReconfirmation' \
    'Self::GenerationRejected' \
    'Self::StaleGeneration' \
    'Self::ReconfirmationNotRequired' \
    'Self::GenerationExhausted'; do
    grep -Fq "$contract" "$attempt" || {
        echo "Robot credentials: missing fail-closed attempt contract $contract" >&2
        exit 1
    }
done

default_tree="$(
    cargo tree --locked -p cloud-sdk-hetzner --no-default-features \
        --edges normal --prefix none
)"
if printf '%s\n' "$default_tree" | grep -Eq '^(base64-ng|reqwest|tokio|hyper|rustls) v'; then
    echo "Robot credentials: transport or encoding entered the default provider graph" >&2
    exit 1
fi

scripts/check_robot_wire_fixture.py
scripts/test-robot-wire-fixture.py

echo "Robot credentials are scoped, protected, generation-bound, and lockout-aware."
