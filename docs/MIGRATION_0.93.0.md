# Migrating Source Users To v0.93.0

v0.93.0 is an additive internal milestone. It is tagged after review but is
not separately published to crates.io. Source users can select the exact tag:

```toml
[dependencies]
cloud-sdk = { git = "https://github.com/valkyoth/cloud-sdk", tag = "v0.93.0" }
cloud-sdk-hetzner = { git = "https://github.com/valkyoth/cloud-sdk", tag = "v0.93.0", version = "0.45.0", features = ["serde"] }
```

## Billable Orders

The new `robot::ordering` mutation types cover standard servers, Server
Auction products, and per-server addons. The required sequence is:

1. Fetch and strictly decode the current product and account currency through
   `execute_observed_blocking`, `execute_observed_async`, or
   `execute_observed_local_async`. These paths produce non-forgeable
   `CredentialObserved<T>` values and reject credential replacement during a
   read.
2. Build the catalog-derived typed plan from matching product and currency
   observations.
3. For addon orders, construct `RobotRipeReason` and the catalog-type-specific
   `RobotAddonOrderParameters`; IPv4-related types require a reason and only
   `subnet_ipv4` accepts an optional IPv4 gateway.
4. Construct the corresponding create request with an explicit scale-4 gross
   recurring-plus-setup spending ceiling.
5. Prepare through `PreparationStorageGuard`.
6. Derive `RobotOrderAuthorizationEvidence::for_request` from the exact order
   request, then confirm endpoint, account identity, context, validity, replay policy,
   attempt budget, and an idempotency identity only for reconcile-and-retry.
7. Build a digest fingerprint. Sensitive order bodies reject the direct
   canonical fingerprint route.
8. Call `RobotOrderPlanFingerprintDigest::mint_permit` once, begin one attempt,
   and execute through the same credential-bound, delivery-classified transport.
9. Decode the strict `201` transaction and verify observable order intent.

There is no automatic retry. A proven `NotSent` result follows explicit
recovery policy. `PossiblySent`, `ResponseStarted`, or an abandoned attempt
requires a fresh exact plan and a transaction snapshot produced by an observed
authenticated execution with the same credential. Standard addon matching is
order-independent and quantity-preserving. Auction creation rejects any
unrequested addon, and addon creation requires the exact catalog price and
documented product type.
Historical addon reconciliation intentionally uses only the request-bound
server and product identity. A matching transaction blocks retry even when its
price or optional type differs, because treating that mismatch as absence could
authorize a duplicate charge.
Do not reuse a transaction snapshot captured before the uncertain attempt.
Robot exposes only a 30-day list and no revision token, so ambiguous history
must be resolved by an operator rather than retried optimistically.

## Compatibility

No v0.92 API changed. The v0.93 implementation-stop API was tightened before
tagging: addon construction now requires typed parameters, authorization
evidence replaces a standalone account argument, and permits are minted from a
strong digest rather than from a copyable subject. The workspace MSRV remains
Rust 1.92.0, default features remain empty, and no dependency was added. The
first crates.io package containing this cumulative work remains v0.95.0.
