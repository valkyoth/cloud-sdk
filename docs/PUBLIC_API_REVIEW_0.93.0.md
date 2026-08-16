# Public API Review 0.93.0

Status: implementation stop; incremental pentest required.

## Added Surface

`cloud_sdk_hetzner::robot::ordering` adds three catalog-derived create request
types, guarded prepared/checked wrappers, typed order failures, one account
scope, cost errors, typed plan fingerprints, a direct non-cloneable cost
permit, attempt authority, and request-bound absent-transaction proof.

All mutation APIs require `serde` because catalog and transaction models use
bounded owned decoding. Default features remain empty. No raw
`PrepareOperation`, public untyped prepared accessor, raw decoder, high-level
client, custom endpoint, or automatic retry route was added.

## Authority Review

- Catalog plans retain product, exact observed price, currency, selectors,
  and quantities.
- Request construction rejects a gross first-invoice amount above the caller
  ceiling.
- Fingerprints cover exact request bytes, endpoint, account, cost, validity,
  replay, budget, context, and reconciliation identity.
- Sensitive request bodies require digest fingerprinting.
- `RobotOrderCostPermit` is neither `Copy` nor `Clone`; attempts retain exact
  request provenance and expose no prepared request.
- Single-attempt plans reject idempotency identity. Reconcile-and-retry plans
  require one through the shared plan contract.
- Uncertain delivery cannot return to ready state without a same-request
  absent-transaction proof and matching fresh subject.

## Compatibility

The change is additive pre-1.0 API. `cloud-sdk` advances to `0.93.0` for the
internal tag. `cloud-sdk-hetzner` remains at published package version
`0.45.0` until the v0.95.0 cumulative checkpoint.
