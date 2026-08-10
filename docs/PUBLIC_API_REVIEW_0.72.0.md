# v0.72.0 Public API Review

Status: implementation stop reached; pentest required before tagging.

Scope: changes from signed v0.71.0 through the v0.72.0 implementation stop.

## Provider-Neutral Workspace API

v0.72 adds no provider-neutral runtime API. The `cloud-sdk` source version
advances with the milestone and retains its documented Rust and `no_std`
boundaries.

## Hetzner Security Client API

With `serde`, `cloud_sdk_hetzner::client` adds:

- one named blocking, `Send` async, and local-async method for each of the seven
  active read-only certificate and SSH-key operations;
- one named cleanup-owning preparation method and three permit-authorized
  execution methods for each of the five mutation and two destructive
  operations;
- `SecurityReadResult<E>` for complete checked read execution;
- `SecurityClientMethodDescriptor` and `SECURITY_CLIENT_METHODS` for exhaustive
  policy inspection.

The exact 14-operation surface is generated from the reviewed operation
association manifest. Methods exist only on
`HetznerClient<T, SecurityService, OfficialEndpointTrust>` and preserve exact
pagination, action, response, retry, and permit classifications.

## Compatibility

Existing request domains, generic associated execution, checked response
models, Cloud and DNS client methods, custom-client construction, and transport
adapters remain available. No default feature or dependency changes.
