#!/usr/bin/env sh
set -eu

client_dir="crates/cloud-sdk/src/client"

for required in \
    'pub trait ClientOperation' \
    'pub struct ClientKernel' \
    'pub struct ClientWorkspace' \
    'pub struct ClientWorkspacePool' \
    'pub struct ClientWorkspaceLease' \
    'pub struct ClientResponse'; do
    if ! grep -R -Fq "$required" crates/cloud-sdk/src/client.rs "$client_dir"; then
        echo "client kernel: missing required contract: $required" >&2
        exit 1
    fi
done

for mode in execute_blocking execute_async execute_local_async; do
    if ! grep -Fq "fn $mode" "$client_dir/execution.rs"; then
        echo "client kernel: missing execution mode $mode" >&2
        exit 1
    fi
done

if ! grep -Fq '> + Send' "$client_dir/execution.rs"; then
    echo "client kernel: Send async execution does not promise a Send future" >&2
    exit 1
fi

if grep -Eq 'Vec|Box|Mutex|Condvar|VecDeque|spawn|sleep' "$client_dir/workspace.rs"; then
    echo "client kernel: workspace admission gained allocation, queuing, or runtime policy" >&2
    exit 1
fi

if ! grep -Fq 'AtomicUsize' "$client_dir/workspace.rs" \
    || ! grep -Fq 'WorkspaceAcquireError::Exhausted' "$client_dir/workspace.rs"; then
    echo "client kernel: bounded atomic admission contract is incomplete" >&2
    exit 1
fi

if grep -A18 -F 'pub trait ClientOperation' crates/cloud-sdk/src/client.rs \
    | grep -Fq 'PreparedRequest'; then
    echo "client kernel: decoder can recover a reusable prepared request" >&2
    exit 1
fi

for helper in send_blocking send_async send_local_async; do
    if ! grep -R -Eq "pub\(crate\).*fn $helper" crates/cloud-sdk/src/operation; then
        echo "client kernel: raw send-once helper $helper is absent or public" >&2
        exit 1
    fi
done

if ! grep -Fq 'pub trait ResponseStorageSanitizer: Sync' \
    crates/cloud-sdk/src/transport/cleanup.rs; then
    echo "client kernel: response sanitizer can make the Send path non-Send" >&2
    exit 1
fi

for test_name in \
    workspace_pool_rejects_invalid_bounds_and_reuses_released_slots \
    every_execution_mode_uses_the_same_error_decoder_and_cleanup \
    endpoint_and_auth_mismatch_fail_closed_and_clear \
    mutation_without_a_permit_never_reaches_transport \
    cancelled_async_request_releases_slot_and_clears_every_buffer; do
    if ! grep -R -Fq "fn $test_name" "$client_dir/tests"; then
        echo "client kernel: missing regression $test_name" >&2
        exit 1
    fi
done

cargo test -p cloud-sdk client --all-features
cargo test -p cloud-sdk --doc --all-features
cargo check -p cloud-sdk --no-default-features

echo "Provider-generic client kernel checks passed."
