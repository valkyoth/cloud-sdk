#!/usr/bin/env sh
set -eu

if [ -n "${CLOUD_SDK_HETZNER_TOKEN_FILE:-}" ] ||
    [ -n "${CLOUD_SDK_HETZNER_ROBOT_USERNAME_FILE:-}" ] ||
    [ -n "${CLOUD_SDK_HETZNER_ROBOT_PASSWORD_FILE:-}" ] ||
    [ -n "${CLOUD_SDK_HETZNER_ALLOW_DESTRUCTIVE:-}" ]; then
    echo "controlled mutation: credential and live opt-in variables are forbidden" >&2
    exit 2
fi

cargo test --locked -p cloud-sdk-hetzner --all-features \
    --test vertical_execution action_and_no_content_slices_cross_permit_and_executor_paths \
    -- --exact
cargo test --locked -p cloud-sdk-hetzner --all-features \
    --test dns_client named_dns_mutation_requires_a_bound_permit_and_decodes_its_action \
    -- --exact
cargo test --locked -p cloud-sdk-hetzner --all-features \
    --test security_client uploaded_private_key_is_redacted_permitted_and_cleanup_owned \
    -- --exact
cargo test --locked -p cloud-sdk-hetzner --all-features \
    --test storage_client password_reset_is_redacted_digest_bound_permitted_and_cleanup_owned \
    -- --exact
cargo test --locked -p cloud-sdk-hetzner --all-features \
    --test robot_client mutation_permit_unauthorized_response_spends_authority_and_locks_credentials \
    -- --exact
scripts/check_robot_order_mutations.sh

echo "Credential-free mutation safety qualification passed."
