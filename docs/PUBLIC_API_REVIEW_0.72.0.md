# v0.72.0 Public API Review

Status: release candidate; pentest and final retest passed.

Scope: changes from signed v0.71.0 through the v0.72.0 implementation stop.

## Provider-Neutral Workspace API

`PreparedRequest` adds `RequestBodySensitivity`, `with_sensitive_body`, and
`body_sensitivity`. Construction requires an explicit sensitivity value; there
is no public default. Providers use this metadata to prevent long-lived exact
canonical copies of secret-bearing bodies. `with_sensitive_body` remains a
one-way upgrade for reviewed wrappers and cannot downgrade a classification.
`build_canonical_plan` and
`build_canonical_fingerprint` return the new payload-free
`SensitiveBodyRequiresDigest` error for those requests; their digest variants
remain available and clear canonical scratch after hashing.

Sensitivity is part of prepared retry-policy equality and canonical plan or
retry identity. The API remains allocation-free and `no_std` with no new core
dependency.

Hetzner's sealed `BodyWire` contract likewise has no default. Every generated
or manual body adapter must choose `public` or provide a classifier. Dynamic
classifiers cover server user data, TSIG-bearing nameservers, optional
zonefiles, and uploaded certificate keys. Storage Box passwords, imported
zonefiles, and RRSet record values or comments are always sensitive.

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
- `Sha256PlanHasher`, backed by the existing allocation-free `sha2` dependency,
  for sensitive plan fingerprints.

The exact 14-operation surface is generated from the reviewed operation
association manifest. Methods exist only on
`HetznerClient<T, SecurityService, OfficialEndpointTrust>` and preserve exact
pagination, action, response, retry, and permit classifications.

## Compatibility

Direct `PreparedRequest::new` callers must add the explicit
`RequestBodySensitivity` argument. Existing request domains, generic associated
execution, checked response models, Cloud and DNS client methods, custom-client
construction, and transport adapters remain available. Exact fingerprints
continue to work for bodies not marked sensitive. No default feature or
dependency changes.
