# v0.70.0 Public API Review

Status: implementation complete; pentest required.

Scope: changes from signed v0.69.0 through v0.70.0, with cumulative publication
review from v0.65.0.

## Provider-Neutral Workspace API

v0.70 adds no provider-neutral runtime API. The `cloud-sdk` package advances to
the public checkpoint and includes the reviewed core changes accumulated in
v0.66-v0.69, including complete workspace profiles from v0.69.

## Hetzner Cloud Client API

With `serde`, `cloud_sdk_hetzner::client` adds:

- one named blocking, `Send` async, and local-async method for every active
  read-only Cloud operation;
- one named cleanup-owning preparation method and three permit-authorized
  execution methods for every mutation, destructive, and cost operation;
- `CloudReadResult<E>` for the complete read execution result;
- `CloudClientMethodDescriptor` and `CLOUD_CLIENT_METHODS` for exhaustive
  source-locked method inventory and policy inspection.

The method set contains exactly 139 operations. It is generated from the
reviewed operation associations, not maintained as an independent hand-written
classification. Named methods remain available only on
`HetznerClient<T, CloudService, OfficialEndpointTrust>`.

State-changing Send-async and local-async methods are ordinary functions that
return opaque futures rather than `async fn`. This source-compatible call and
`.await` shape is required so complete response storage is cleared during
future construction, including when callers drop the future unpolled. The
typed association and provider-neutral permit wrappers use the same eager
construction boundary.

## Compatibility

Existing generic associated preparation, execution, decoding, and permit APIs
remain available. No default feature changed. `cloud-sdk-hetzner` remains
allocation-free, transport-free, and `no_std` without optional features.

This release does not add named DNS, security, Console Storage Box, Robot, or
custom-endpoint execution methods. Those scopes remain explicit roadmap work.
