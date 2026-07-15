# Migrating To v0.29.0

The workspace facade moves to `0.29.0`. `cloud-sdk-reqwest` moves to `0.20.0`
and `cloud-sdk-testkit` moves to `0.18.0`. The Hetzner provider receives a
dependency-only patch at `0.22.1`, and `cloud-sdk-sanitization` receives a
dependency-only patch at `0.13.15`.

## Prepared Operations

`cloud-sdk::operation` adds the provider-neutral preparation and response
policy contract. Implement `PrepareOperation` for typed provider operations,
write request targets and bodies into `PreparationStorage`, and return a
`PreparedRequest` containing:

- the complete `TransportRequest`;
- a `ProviderService` with immutable expected endpoint identity;
- explicit `OperationMetadata`; and
- a complete `ResponsePolicy`.

No operation metadata implements `Default`. Read-only operations must be
`Safe`, mutations and destructive operations cannot be `Safe`, and
non-idempotent operations cannot be marked retry eligible. Retry eligibility
only admits caller-owned policy; execution still sends exactly once.

The common contract is available in v0.29. Existing Hetzner request models do
not all implement it until the planned v0.30 operation pass, so integrations
may continue using `TransportRequest` directly during this transition.

## Checked Responses

`PreparedRequest::execute_blocking` and `execute_async` now provide the common
secure execution path. They verify the bound transport endpoint before send,
lend at most the configured response capacity, and return `CheckedResponse`
only after expected status, body policy, body limit, and content-type policy
pass.

Prepared transports must additionally implement `ResponseStorageSanitizer`.
Execution invokes it for the complete caller-owned response buffer before
endpoint verification and before lending the smaller admitted slice to
`send`. Production implementations must use non-elidable cleanup. The reqwest
adapters use the reviewed `cloud-sdk-sanitization` boundary; this prevents a
smaller operation policy from retaining bytes written by an earlier larger
response in the unused tail.

`TransportResponse` now exposes `content_type() -> Option<ResponseContentType>`.
Custom transports should validate and attach one bounded response content type
with `with_content_type`. Missing metadata remains `None`; malformed metadata
must be a transport error rather than an unchecked value.

## Reqwest Behavior

Both reqwest adapters capture a valid response `Content-Type`. Duplicate,
non-textual, oversized, or malformed values now return
`TransportError::InvalidResponseContentType` before response bytes are exposed.
Servers that emitted syntactically invalid media parameters must correct their
headers.

## Testkit Behavior

`MockTransport::with_endpoint` binds endpoint identity for prepared execution.
An unbound mock returns `EndpointIdentityError::UnboundTransport` through the
prepared execution boundary.

`ExpectedRequest::with_content_type` opts into an exact request content-type
expectation. `ResponseFixture::with_content_type` supplies response metadata,
and invalid fixture values fail as `MockError::InvalidFixtureMetadata`.
`PreparedRequestRecord::capture` records redacted request shape plus complete
service, operation, and response policy for assertions.

## Unchanged Boundaries

- Default features remain empty.
- `cloud-sdk`, `cloud-sdk-hetzner`, and `cloud-sdk-testkit` remain `no_std` and
  allocation-free by default.
- No new third-party dependency is admitted.
- The core owns no network client, TLS stack, runtime, filesystem, credential
  store, retry loop, sleep, clock, queue, or executor.
