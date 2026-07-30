# Migrating To v0.41

v0.41 separates credential-free transport from mandatory bearer
authentication policy.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.41.0"
cloud-sdk-hetzner = "0.32.2"
cloud-sdk-reqwest = { version = "0.28.0", features = ["blocking-rustls"] }
cloud-sdk-sanitization = "0.16.0"
cloud-sdk-testkit = "0.24.1"
```

## Authenticated Client Construction

`BlockingClientBuilder::new` and `AsyncClientBuilder::new` now require a
`BearerCredential` rather than a bare `BearerToken`. Bind the token to an
immutable `BearerCredentialScope::new(provider, service, endpoint)`. Those
three dimensions are mandatory; audience, account, and tenant remain optional
provider-owned additions. Client construction rejects a configured endpoint
that differs from the credential endpoint.

Every send uses `AuthenticatedRequest::new(request, policy)`. The policy must
mark provider, service, and endpoint `Required` with exact values and state
`Required`, `Optional`, or `Forbidden` for the remaining fields.
Authenticated clients now implement `BlockingAuthenticatedTransport` or
`AsyncAuthenticatedTransport`; they no longer implement the credential-free
transport traits.

This is intentionally breaking. It prevents an authenticated client from
sending a request that omitted its provider or operation-owned scope policy.
The raw clients remain credential-free and continue implementing the raw
executor traits.

## Rotation And Refresh

Rotation methods now return `CredentialGeneration`. Obtain a redacted
`BearerCredentialSnapshot` and call `refresh_handoff` before external refresh
work. The resulting `BearerRefreshHandoff` is bound to that credential store.
Install the result with `refresh_bearer_token` or a source-clearing variant. A
foreign handoff returns `TokenRefreshError::CredentialMismatch`; a stale
handoff returns `TokenRefreshError::StaleGeneration`.

Prefer mutable-byte or `SecretBuffer` ingestion when the source can be
cleared. The immutable `BearerToken::new(&str)` compatibility constructor
cannot clear caller-owned text.

## Hetzner Migration Boundary

The v0.41 Hetzner live-smoke harness supplies exact provider, service, and
official endpoint policy. The 208 prepared operations migrate to
provider-owned policy construction on the complete new wire/authentication
path in v0.43. Until then, application code using the authenticated adapter
must construct the matching required policy explicitly; v0.41 does not
describe that application-level assembly as provider-type-enforced.

See [`AUTHENTICATION_POLICY.md`](AUTHENTICATION_POLICY.md) for the complete
scope, lifecycle, cleanup, and non-guarantee contract.
