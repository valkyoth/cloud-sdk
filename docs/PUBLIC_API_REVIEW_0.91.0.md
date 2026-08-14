# Public API Review 0.91.0

Status: implementation stop; pentest required.

## Added Surface

`cloud_sdk_hetzner::robot` now exposes all six active read-only ordering-
catalog requests: standard products, one standard product, Server Auction
products, one auction product, per-server addon products, and account
currency.

The `serde` feature adds bounded strict response models, exact non-floating-
point prices, protected product/location/choice/text values, request-associated
prepared and checked exchanges, and operation-specific Robot failure codes.
Catalog-derived standard, auction, and addon plan types retain references to
the decoded snapshot and always expose
`RobotCatalogPriceWarning::RevalidateImmediatelyBeforePurchase`.
`RobotAddonCatalog<'request>` retains the exact per-server request through
decoding. Addon plans select a product by index from that catalog and cannot
accept or substitute a separate server identity.
Validated plan selections are retained as direct references, so public
accessors do not need an impossible-state panic path.

## Non-Execution Boundary

No ordering plan implements `PrepareOperation`, contains a transport request,
or serializes a purchase body. The six operations are `GET` requests marked
read-only and safe. Retry remains an explicit caller-policy decision even for
these reads. Billable ordering mutations remain reserved for v0.93.0.

## Compatibility

This is additive pre-1.0 API. `cloud-sdk` advances to `0.91.0` for source and
tag identity. `cloud-sdk-hetzner` remains `0.45.0` until the v0.95 cumulative
publication checkpoint. Existing provider-neutral transport, authentication,
client, retry, permit, and published provider behavior is unchanged.

## Review Result

Prices preserve their exact source text and compare numerically without
floating point. Standard-product filters reject inverted monthly or setup
ranges. Lists are bounded and reject duplicate product identities. Detail
responses are bound to the requested product ID. Location-specific product and
addon prices must refer to advertised locations, and hourly net/gross auction
prices must be present or absent together. Returned provider text is protected,
redacted, bounded, and available only through closure-scoped inspection. Plan
selection diagnostics are completely redacted, and exact decimal scalar
mirrors receive best-effort volatile cleanup on drop.
