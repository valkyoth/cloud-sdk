# Public API Review 0.92.0

Status: implementation stop; incremental pentest required.

## Added Surface

`cloud_sdk_hetzner::robot` now exposes standard-server, Server Auction, and
per-server addon transaction list/detail requests. All six implement
`PrepareOperation`; the `serde` feature adds request-associated prepared and
checked exchanges, strict owned response models, and one source-locked
`RobotOrderTransactionFailureCode::NotFound` category.

`RobotOrderTransactionQuota` and `ROBOT_ORDER_TRANSACTION_QUOTA` expose the
single source-locked account allowance of 500 requests per hour. Every request
has a `quota()` accessor returning that same value; it does not represent six
independent budgets.

New protected value and model types retain transaction IDs, timestamps, finite
status, server identities, key metadata, product snapshots, exact addon
prices, and resulting resources. List wrappers expose bounded snapshots from
Robot's documented fixed 30-day window.

## Non-Execution Boundary

Every request is `GET`, read-only, safe, no-known-cost, and explicitly retried
only under caller policy. No new type serializes a form, accepts a cost permit,
implements a purchase, or turns a catalog plan into an executable operation.
Robot ordering mutations remain reserved for v0.93.0.

## Compatibility

This is additive pre-1.0 API. `cloud-sdk` advances to `0.92.0` for source and
tag identity. `cloud-sdk-hetzner` remains `0.45.0` until the v0.95 cumulative
publication checkpoint. Existing provider-neutral and published provider
behavior is unchanged.

## Review Result

Detail decoding is tied to the exact protected request ID. Standard and auction
transactions expose a server identity only in `ready` state. All text and
collections are bounded, diagnostics are redacted, timestamps are calendar-
validated, and duplicate transaction, key, addon, and resource identities fail
closed. Production preparation maps target and prepared-policy invariant
failures into existing payload-free error variants instead of panicking.
