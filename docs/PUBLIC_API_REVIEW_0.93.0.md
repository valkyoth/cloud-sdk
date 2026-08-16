# Public API Review 0.93.0

Status: implementation stop; incremental pentest required.

## Added Surface

`cloud_sdk_hetzner::robot::ordering` adds three catalog-derived create request
types, guarded prepared/checked wrappers, typed order failures, credential-
bound catalog and transaction observations, typed RIPE addon parameters, cost errors, typed plan
fingerprints, a one-shot direct non-cloneable cost permit, creation-specific
auction response types, attempt authority, and request-bound absent-
transaction proof.

All mutation APIs require `serde` because catalog and transaction models use
bounded owned decoding. Default features remain empty. No raw
`PrepareOperation`, public untyped prepared accessor, raw decoder, high-level
client, custom endpoint, or automatic retry route was added.

## Authority Review

- Catalog plans require transport-produced product and currency observations
  from one credential lifecycle and retain exact price, selectors, and quantities.
- Request construction rejects a gross first-invoice amount above the caller
  ceiling.
- Fingerprints cover exact request bytes, endpoint, account, credential
  lineage, cost, validity, replay, budget, context, and reconciliation identity.
- Sensitive request bodies require digest fingerprinting.
- `RobotOrderPlanFingerprintDigest::mint_permit` can mint only once.
  `RobotOrderCostPermit` is neither `Copy` nor `Clone`; attempts retain exact
  request and credential provenance and expose no prepared request.
- Single-attempt plans reject idempotency identity. Reconcile-and-retry plans
  require one through the shared plan contract.
- Uncertain delivery cannot return to ready state without a same-request
  same-credential observed absent-transaction proof and matching fresh subject.
- Auction creation and transaction-history responses have distinct types and
  exact schemas. Auction creation forbids unrequested addons. Addon creation
  requires its provider type and exact catalog price; documented GET responses
  may omit the type. Historical addon reconciliation conservatively matches
  server and product identity without using mutable price or optional type
  fields to authorize another potentially billable attempt.

## Compatibility

The change is additive pre-1.0 API. `cloud-sdk` advances to `0.93.0` for the
internal tag. `cloud-sdk-hetzner` remains at published package version
`0.45.0` until the v0.95.0 cumulative checkpoint.
