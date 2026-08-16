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

1. Fetch and strictly decode the current product and account currency.
2. Build the catalog-derived typed plan.
3. Construct the corresponding create request with an explicit scale-4 gross
   recurring-plus-setup spending ceiling.
4. Prepare through `PreparationStorageGuard`.
5. Confirm endpoint, account identity, context, validity, replay policy,
   attempt budget, and an idempotency identity only for reconcile-and-retry.
6. Build a digest fingerprint. Sensitive order bodies reject the direct
   canonical fingerprint route.
7. Mint `RobotOrderCostPermit`, begin one attempt, and execute through a
   delivery-classified authenticated transport.
8. Decode the strict `201` transaction and verify observable order intent.

There is no automatic retry. A proven `NotSent` result follows explicit
recovery policy. `PossiblySent`, `ResponseStarted`, or an abandoned attempt
requires a fresh exact plan and caller-performed transaction reconciliation.
Do not reuse a transaction snapshot captured before the uncertain attempt.
Robot exposes only a 30-day list and no revision token, so ambiguous history
must be resolved by an operator rather than retried optimistically.

## Compatibility

No v0.92 request constructor or response accessor changed. The workspace MSRV
remains Rust 1.92.0, default features remain empty, and no dependency was
added. The first crates.io package containing this cumulative work remains the
planned v0.95.0 checkpoint.
