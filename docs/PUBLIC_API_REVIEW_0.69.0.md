# v0.69.0 Public API Review

Status: implementation complete; pentest required.

Scope: changes from signed v0.68.0 through v0.69.0.

## Provider-Neutral Workspace API

`cloud_sdk::client` adds `ClientCapacityProfile` and
`ClientCapacityError`. The `EMBEDDED`, `DEFAULT`, and `LARGE` profiles cover all
four client regions together. `ClientWorkspace::for_profile` clears every
supplied region before validation and rejects the first insufficient capacity.

The optional `alloc` feature adds `OwnedClientWorkspace`, which fallibly
allocates an exact profile, lends it as an ordinary `ClientWorkspace`, and
clears all four complete allocations on drop. Existing
`ClientWorkspace::new` behavior and the empty default feature graph are
unchanged.

## Hetzner Client API

`cloud_sdk_hetzner::client` adds:

- `HetznerClient<T, S, E>` plus Cloud, DNS, security, and Storage aliases;
- exact official constructors for all four service identities;
- explicitly acknowledged custom HTTPS constructors;
- public official/custom trust markers and inspectable trust provenance;
- shared read-only blocking, `Send` async, and local async execution when the
  `serde` feature is enabled.

The service is represented in the client type. The new
`HetznerClientOperation` trait is implemented only for associated read-only
operations, is sealed against foreign implementations, and retains their
provider service. This makes forged and cross-service client execution a
compile-time mismatch. Associated mutations and destructive or cost-bearing
operations do not gain the direct client execution contract.

Custom clients carry `CustomEndpointTrust`; only official-trust clients expose
execution methods in v0.69. The separate marker prevents an explicitly trusted
custom credential destination from weakening source-locked official operation
policies.

## Compatibility

All additions are backward compatible for source users. Default builds remain
allocation-free and `no_std`. Enabling `serde` now gives read-only associated
operations provider-neutral `PrepareOperation` and `ClientOperation`
implementations; no existing method or response type changed.
