# Rejected Abstractions 0.93.0

Status: implementation stop; incremental pentest required.

## Generic Mutation Permit

Returning a provider-neutral `CostPermit` directly was rejected because it
would lose the concrete Robot request association needed by checked response
decoding and transaction reconciliation. The Robot wrapper remains sealed and
retains request provenance through planning, execution, and recovery.

## Automatic Retry

Automatic retry was rejected. Robot order creation is non-idempotent and a
transport failure may occur after the provider accepted the order. Delivery
classification is preserved, and uncertain outcomes require reconciliation.

## Caller-Asserted Reconciliation

A boolean or untyped "not applied" flag was rejected. The only accepted proof
is minted by comparing the exact request with a bounded typed transaction
list. Matching and identical historical transactions fail closed.

## CI Test Orders

Using Robot's `test=true` input in CI was rejected. It would still require real
credentials and trust provider-side non-processing semantics. CI uses only
deterministic local fixtures and is statically checked for absence of billable
routes.

## Floating-Point Cost

Floating-point aggregation and caller-entered prices were rejected. Cost is
derived from strict catalog decimals as exact scale-4 integer units, includes
gross recurring and setup amounts, and fails on overflow or ceiling breach.

## Unsafe Lifetime Recovery

Unsafe extraction of borrowed prepared requests from cleanup guards was
rejected. Stable Rust's late-error borrow limitation is handled with
prevalidation and guard-owned cleanup, preserving `#![forbid(unsafe_code)]`.
