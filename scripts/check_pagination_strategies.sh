#!/usr/bin/env sh
set -eu

facade=crates/cloud-sdk/src/pagination.rs
link=crates/cloud-sdk/src/pagination/link.rs
target=crates/cloud-sdk/src/transport/request_target.rs
provider_link_query=crates/cloud-sdk/src/transport/request_target/provider_link.rs

for required in \
    'pub struct PaginationLimits' \
    'pub struct PaginationBudget' \
    'pub struct NumberedPagination' \
    'pub struct OffsetPagination' \
    'pub struct PaginationCursor' \
    'pub struct PaginationMarker' \
    'pub struct CursorHistory' \
    'pub struct ValidatedProviderLink'; do
    if ! grep -R -Fq "$required" crates/cloud-sdk/src/pagination; then
        echo "pagination strategies: missing contract $required" >&2
        exit 1
    fi
done

for required in \
    'RequestTarget::from_provider_link' \
    'endpoint: EndpointIdentity' \
    'execute_blocking' \
    'execute_async' \
    'send_authenticated' \
    'endpoint_identity()' \
    'ProviderLinkMethodChanged' \
    'ProviderLinkOperationChanged' \
    'ProviderLinkPathChanged'; do
    if ! grep -Fq "$required" "$link" "$facade"; then
        echo "pagination strategies: missing provider-link check $required" >&2
        exit 1
    fi
done

for required in \
    'pub struct ProviderLinkQuery' \
    'ProviderLinkQueryCannotAssemble' \
    'pub(crate) fn from_provider_link'; do
    if ! grep -Fq "$required" "$target" "$provider_link_query"; then
        echo "pagination strategies: missing raw-link target boundary $required" >&2
        exit 1
    fi
done

cargo test --locked -p cloud-sdk --all-features pagination
cargo test --locked -p cloud-sdk --all-features transport::request_target
cargo test --locked -p cloud-sdk --doc
cargo test --locked -p cloud-sdk-hetzner --all-features pagination
cargo check --locked -p cloud-sdk --no-default-features
cargo check --locked --manifest-path fuzz/Cargo.toml \
    --bin pagination \
    --bin pagination_opaque \
    --bin provider_links

printf '%s\n' 'Pagination strategy, raw-link, cleanup, and no_std checks passed.'
