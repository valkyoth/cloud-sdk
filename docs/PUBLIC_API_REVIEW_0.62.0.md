# v0.62.0 Public API Review

Status: implementation stop pending pentest.

Scope: changes from signed v0.61.0 through v0.62.0.

## Freeze Decision

The OVHcloud v2 probe and the unchanged Robot wire fixture require no new
provider-neutral exception. The existing identities, endpoint policies,
authentication lifetimes, prepared requests, permits, response guards,
pagination, incremental JSON, blocking, Send-async, and local-async contracts
are therefore frozen for pre-1.0 provider completion.

## Added Provider API

- `HetznerSuccess::{Locations, Certificate, StorageBoxes}` and their complete
  selected-operation model types.
- `decode_associated_response`, which preserves an exact operation marker until
  the checked decoder consumes the prepared request.
- protected multiline validation for certificate PEM and DNS zonefile output;
  diagnostics continue to use stricter single-line validation.

The models are `#[non_exhaustive]`; provider additions can be represented
without permitting downstream construction. All selected required fields,
nullability, bounds, booleans, nested associations, and pagination are checked.

## Added Testkit API

`ResponseFixture::success_at` models exact successful provider statuses.
`MockError` now implements `DeliveryClassified` as `NotSent`, which is sound
because testkit performs no peer I/O. These additions let mutation and
destructive permit fixtures exercise `201` and `204` responses.

## Compatibility

No default feature, runtime dependency, executor, network client, filesystem,
clock, TLS, or secret-store dependency is added. The default graph remains
`no_std`; complete models remain behind the existing `serde`/`alloc` feature.
