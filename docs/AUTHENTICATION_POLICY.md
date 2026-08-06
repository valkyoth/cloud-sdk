# Authentication Policy

Status: v0.58 regional authority and expiring OAuth conformance is at its
implementation stop; pentest required.

The authentication layer is separate from credential-free raw HTTP execution. Core
defines scope, generation, caller-clock credential lifetimes, and authenticated
transport contracts without owning token bytes, acquisition, clocks,
executors, or secret stores.
`cloud-sdk-reqwest` owns optional bearer and Basic credential lifecycles and
header construction.

## Ownership

Exactly one layer may construct `Authorization`:

- `RequestHeader` rejects caller-supplied authorization fields.
- raw HTTP executors have no credential input and add no authorization;
- authenticated reqwest clients inject one sensitive bearer or Basic field only after
  endpoint and complete scope policy validation;
- authenticated clients do not implement the credential-free
  `BlockingTransport` or `AsyncTransport` traits.

Use raw clients only below a separately reviewed authentication layer. Use
`BlockingAuthenticatedTransport` or `AsyncAuthenticatedTransport` when the
reqwest adapter owns a credential.

Every internal `AuthenticatedRequest` also carries a complete
`RawResponsePolicy`. Construction and extraction are crate-private; application
and provider code initiates authenticated I/O through checked
`PreparedRequest::execute_*`, permit-authorized execution, or the structurally
GET-only provider-link continuation path. Bearer and Basic clients validate
scope, construct one sensitive authorization field, and then execute through
the same bounded raw Hyper engine as the credential-free raw clients. There is
no high-level reqwest compatibility fallback.

## Credential Scope

`AuthenticationScopePolicy` has an explicit `Required`, `Optional`, or
`Forbidden` rule for each field:

| Field | Purpose |
| --- | --- |
| provider | provider namespace |
| service | provider-owned API or product |
| endpoint | normalized scheme, host, effective port, and base path |
| audience | provider-defined token audience |
| account | provider-defined account or project binding |
| tenant | provider-defined tenant binding |

Required fields must be present and exactly equal. Optional fields may be
absent, but supplied values must equal policy. Forbidden fields must be absent.
The reqwest adapters narrow this general core contract: provider, service, and
endpoint are mandatory in `BearerCredentialScope` and
`BasicCredentialScope`, and the request policy must mark all three `Required`
with exact values. Optional or forbidden base-identity rules fail closed
before credential snapshot or authorization-header construction.

Endpoint rules are HTTPS-only. Client construction requires the credential
endpoint to equal the configured endpoint, and every send independently
requires both the credential endpoint and required policy endpoint to equal
the configured identity. Custom endpoint values are trusted operator
configuration and must never originate from tenant-controlled input.
The test-only numeric HTTP loopback harness projects only the admitted
transport scheme to HTTPS before invoking the same complete core scope
validator; it does not bypass audience, account, or tenant requirements.

Provider-owned audience, account, and tenant values are bounded to 512 visible
ASCII bytes without whitespace or backslash and are redacted from diagnostics.
Credential scope is immutable after client construction. Bearer rotation
cannot silently change it.

Providers with geographic API and token authorities use
`EndpointPairPolicy`. A finite reviewed table binds one canonical region to
one exact HTTPS API identity and one exact HTTPS token identity. Cross-region
combinations, aliases, duplicate identities, unknown regions, and non-HTTPS
entries fail closed. Token acquisition and authenticated API execution do not
follow redirects; a redirect response is an error, never a new credential
destination.

## Basic Credentials

Basic and bearer credentials are distinct types with distinct client builders.
The Basic username is nonempty visible ASCII without spaces or a colon. The
password is nonempty printable ASCII and may contain spaces and colons. This
conservative profile avoids RFC 7617's otherwise ambiguous default character
encoding. Username, password, and complete encoded authorization values have
independent finite caps.

`base64-ng` performs RFC 4648 padded encoding into exact caller-sized
adapter-owned storage. It is optional, uses no default features, and is
enabled only with an explicit reqwest transport feature.

Basic clients do not rotate or retry credentials. Later Robot credential
policy adds lockout-aware attempt generations before Robot operations become
executable. The v0.42 Robot fixture sends no credential and makes no operation
coverage claim.

## Rotation And Refresh

Every admitted credential starts at `CredentialGeneration::INITIAL`. Rotation
atomically installs a new token and advances the generation. Requests take an
`Arc` snapshot under a short-lived read lock and release the lock before
network I/O or `.await`; in-flight requests continue with their original
snapshot.

Static credentials capture a `BearerRefreshHandoff` from a snapshot before
acquisition. Expiring credentials instead require
`refresh_handoff_at(CredentialTimestamp)`. The adapter issues that handoff only
inside the configured refresh window and strictly before expiry. Caller time
before the lifetime observation is rejected as rollback. The adapter binds
every handoff to one credential-store lineage in addition to the core
generation. Installation is compare-and-swap:

1. capture the current generation and opaque store lineage;
2. acquire a replacement outside the SDK;
3. submit the replacement with the captured handoff;
4. reject a foreign lineage before comparing generations;
5. reject it as stale if rotation or another refresh already advanced state.

Generations never wrap. The SDK supplies no refresh task, clock, queue, retry,
or token source.

`CredentialLifetime::from_expires_in` converts a nonzero provider lifetime
through explicit caller-owned monotonic time and an explicit refresh margin.
Zero lifetime, zero refresh margin, a margin at least as long as the lifetime,
and timestamp overflow fail closed. A nonempty `RefreshRequired` interval is
therefore guaranteed before exclusive expiry. Expiring rotation and refresh
install the token and complete replacement lifetime in one locked generation
change; static and expiring lifecycle modes cannot be changed implicitly.

## Secret Lifetime

Prefer `BearerToken::from_mut_bytes` or `from_secret_buffer`. Matching rotation
and refresh methods clear complete caller-owned mutable or guarded source
storage on success and rejection. `BearerToken::new(&str)` cannot clear its
immutable caller-owned source.

Adapter-owned active and retired bearer allocations clear through
`cloud-sdk-sanitization`. Retired tokens remain alive only while an in-flight
snapshot owns them. Basic username/password inputs, the intermediate
`user:password` value, and encoded credential storage also clear. Authorization
header bytes have their own cleanup-owning allocation and clear after the final
header clone drops.

These guarantees do not cover immutable caller copies, allocator internals,
reqwest, TLS, kernel/device buffers, swap, crash dumps, process abort,
deliberately leaked values, or remote systems. Rotate and revoke provider
tokens according to provider policy.

## Failure And Retry

Scope, endpoint, token-validation, stale-generation, and generation-exhaustion
errors are payload-free. Scope failure occurs in `NotSent` before network work.
Once execution begins, authenticated adapter failures preserve `NotSent`,
`PossiblySent`, or `ResponseStarted`. This layer does not retry. Operation
policy must combine that delivery phase with idempotency before any retry
decision.

## Verification

The release checks cover:

- every required, optional, forbidden, omitted, supplied, and mismatched field;
- base-identity downgrade, HTTP, custom-endpoint, and configured-scope confusion;
- scope validation before header and network activity;
- blocking and async bearer/Basic parity;
- Basic RFC vector, colon/charset/individual/aggregate bounds, and type
  separation;
- mutable and guarded source cleanup on success and rejection;
- in-flight snapshots and retired-token cleanup;
- foreign-store, sequential, and concurrent stale refresh races;
- regional authority-pair mismatch, alias, duplicate, and downgrade rejection;
- explicit-time refresh-window, rollback, expiry, and overflow boundaries;
- atomic token-and-lifetime rotation across blocking and async client clones;
- poisoned lock recovery and generation exhaustion;
- cleanup-owned header copies and redacted diagnostics.

Run `scripts/check_bearer_authentication.sh` and
`scripts/check_basic_and_signing.sh` for the focused contracts. OVHcloud
source conformance additionally runs through
`scripts/check_ovhcloud_authority_conformance.sh`.
