#!/usr/bin/env sh
set -eu

core=crates/cloud-sdk/src/operation/permit
prepared=crates/cloud-sdk/src/operation/prepared.rs
provider=crates/cloud-sdk-hetzner/src/association

for required in \
    'pub struct CanonicalPlanFingerprint' \
    'pub struct PlanFingerprintDigest' \
    'pub struct PlanConfirmation' \
    'pub struct SharedPermitState' \
    'pub struct PermitAttempt'; do
    if ! grep -R -Fq "$required" "$core"; then
        echo "execution permits: missing contract $required" >&2
        exit 1
    fi
done

for permit in MutationPermit DestructivePermit CostPermit; do
    grep -Fq "$permit," "$core/direct.rs"
done
for permit in SharedMutationPermit SharedDestructivePermit SharedCostPermit; do
    grep -Fq "$permit," "$core/shared.rs"
done

for required in \
    'cloud-sdk/plan-confirm/v1\0' \
    'ConstantTimeEq' \
    'DigestRollback' \
    'PlanChange::NoOp' \
    'DeliveryPhase::PossiblySent' \
    'PendingReconciliation' \
    'ClockRollback' \
    'StaleGeneration' \
    'IdempotencyMismatch'; do
    if ! grep -R -Fq "$required" "$core"; then
        echo "execution permits: missing fail-closed contract $required" >&2
        exit 1
    fi
done

grep -Fq 'PreparedExecutionError::AuthorizationRequired' "$prepared"
grep -Fq 'pub trait ReadOnlyOperation' "$provider/policy.rs"
grep -Fq 'O: ReadOnlyOperation' "$provider/prepared.rs"
grep -Fq 'read_only_operation!($marker, $permit)' "$provider/markers.rs"

for contract in \
    'Direct permits are neither `Copy` nor `Clone`' \
    'state is `PossiblySent`' \
    '`reconcile_not_applied`' \
    'The SDK has no clock, random source, price feed'; do
    if ! grep -Fq "$contract" docs/EXECUTION_PERMITS.md; then
        echo "execution permits: missing documented boundary $contract" >&2
        exit 1
    fi
done

cargo test --locked -p cloud-sdk --all-features operation::permit
cargo test --locked -p cloud-sdk --doc --all-features
cargo test --locked -p cloud-sdk-hetzner --doc --all-features
cargo test --locked -p cloud-sdk-testkit --all-features prepared
cargo check --locked -p cloud-sdk --no-default-features
cargo check --locked -p cloud-sdk-hetzner --no-default-features

echo 'Plan-confirm permit identity, lifecycle, provider gating, docs, and no_std checks passed.'
