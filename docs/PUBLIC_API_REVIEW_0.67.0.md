# v0.67.0 Public API Review

Status: implementation stop; incremental pentest required.

Scope: changes from signed v0.66.0 through v0.67.0.

## Added Provider API

- `StorageBoxTypePage`, `StorageBoxSnapshot`, `StorageBoxSnapshotStats`,
  `StorageBoxSubaccount`, and `StorageBoxSubaccountAccessSettings` expose the
  complete pinned Console response shapes through read-only accessors.
- `StorageBoxResource` distinguishes a complete created box from the partial
  snapshot and subaccount references returned by create operations.
- `StorageBoxSnapshotReference` and `StorageBoxSubaccountReference` retain the
  exact identifiers supplied by those partial create responses.
- `HetznerSuccess` has dedicated box, type, snapshot, and subaccount singleton
  and list variants. `CompositeResult::storage_box_resource` exposes typed
  Console create results.
- `UtcTimestamp::try_from_string` permits response parsers to transfer an
  already cleanup-owned timestamp allocation after canonical validation.
- Typed associated preparation now retains an opaque expected response
  identity for modeled Storage Box singleton, parent-scoped list, and create
  operations.
- `AssociatedPlanConfirmation`, associated exact/digest fingerprints and
  subjects, direct/shared `Associated*Permit` wrappers, and
  `AssociatedPermitAttempt` preserve that opaque identity through authorized
  state-changing execution.

## Changed Provider API

- `Money`, `Price`, `Deprecation`, `StorageBoxType`, `StorageBox`,
  `StorageBoxPage`, `StorageBoxSnapshot`, `StorageBoxSubaccount`, and their page
  wrappers use private fields and read-only accessors.
- Their dynamic response text uses cleanup-owning storage and aggregate `Debug`
  output is redacted. These types therefore no longer provide ordinary
  `Clone`, field-based construction, or field-based equality.
- Box/type singleton operations and snapshot/subaccount operations no longer
  fall back to generic `Resource` results.
- Storage Box creation no longer exposes its response through the generic
  `CompositeResult::resource` accessor.
- Creation and deprecation times are canonical `UtcTimestamp` values rather
  than unchecked `String` values.
- `decode_associated_response` and `decode_associated_checked_response` reject
  validly shaped Storage Box responses whose resource or parent identity does
  not match the typed endpoint. Explicit `into_untyped` calls discard this
  binding and document that consequence.
- `EndpointWire::expected_response_identity` and the endpoint adapter macro no
  longer provide a default. Every endpoint family must state its identity
  policy explicitly, while ID-bearing variants are source-locked by tests.
- Console aggregate and reference `Debug` output redacts provider resource
  identifiers as well as dynamic text. Dynamic string-bearing aggregates do
  not implement structural equality.

## Compatibility

These are intentional pre-1.0 provider API changes. No provider-neutral API,
default feature, transport, runtime, TLS, filesystem, clock, or secret-store
boundary changed. The allocation-backed response models remain behind the
existing optional `cloud-sdk-hetzner/serde` feature.
