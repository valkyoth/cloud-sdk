# v0.73.0 Public API Review

Status: release candidate; pentest passed with no findings.

Scope: changes from signed v0.72.0 through the v0.73.0 implementation stop.

## Provider-Neutral Workspace API

No provider-neutral public type, trait, feature, dependency, or behavior
changes. `cloud-sdk` advances to the workspace milestone version only.

## Hetzner Storage Client API

With `serde`, `cloud_sdk_hetzner::client` adds:

- one named blocking, `Send` async, and local-async method for each of the 12
  active read-only Console Storage operations;
- one named cleanup-owning preparation method and three permit-authorized
  execution methods for each of the nine mutation, eight destructive, and two
  cost-authorized operations;
- `StorageReadResult<E>` for complete checked read execution;
- `StorageClientMethodDescriptor` and `STORAGE_CLIENT_METHODS` for exhaustive
  policy inspection.

The exact 31-operation surface is generated from the reviewed operation
association manifest. Methods exist only on
`HetznerClient<T, StorageService, OfficialEndpointTrust>` and preserve exact
pagination, response, retry, sensitivity, identity, and permit
classifications.

## Compatibility

Existing Storage request models, generic associated execution, checked
response models, other service clients, custom-client construction, and
transport adapters remain available. Applications may migrate one operation
at a time to the named methods. No default feature or dependency changes.
