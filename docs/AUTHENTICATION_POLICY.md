# Bearer Authentication Policy

Status: v0.41.0 implementation stop reached; pentest required.

The bearer layer is separate from credential-free raw HTTP execution. Core
defines scope, generation, and authenticated transport contracts without
owning token bytes, acquisition, expiry, clocks, executors, or secret stores.
`cloud-sdk-reqwest` owns the optional bearer-token lifecycle and header
construction.

## Ownership

Exactly one layer may construct `Authorization`:

- `RequestHeader` rejects caller-supplied authorization fields.
- raw HTTP executors have no credential input and add no authorization;
- authenticated reqwest clients inject one sensitive bearer field only after
  endpoint and complete scope policy validation;
- authenticated clients do not implement the credential-free
  `BlockingTransport` or `AsyncTransport` traits.

Use raw clients only below a separately reviewed authentication layer. Use
`BlockingAuthenticatedTransport` or `AsyncAuthenticatedTransport` when the
reqwest adapter owns a bearer credential.

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
Every mismatch fails before token snapshot or authorization-header
construction. An unscoped credential therefore cannot bypass a provider or
operation that requires scope.

Endpoint rules are HTTPS-only. The adapter additionally requires the policy
endpoint to equal its configured endpoint identity. Custom endpoint values are
trusted operator configuration and must never originate from tenant-controlled
input.

Provider-owned audience, account, and tenant values are bounded to 512 visible
ASCII bytes without whitespace or backslash and are redacted from diagnostics.
`BearerCredentialScope` is immutable after client construction. Token rotation
cannot silently change credential scope.

## Rotation And Refresh

Every admitted credential starts at `CredentialGeneration::INITIAL`. Rotation
atomically installs a new token and advances the generation. Requests take an
`Arc` snapshot under a short-lived read lock and release the lock before
network I/O or `.await`; in-flight requests continue with their original
snapshot.

External refresh logic captures a `RefreshHandoff` from a snapshot before
acquisition. Installation is compare-and-swap:

1. capture the current generation;
2. acquire a replacement outside the SDK;
3. submit the replacement with the captured handoff;
4. reject it as stale if rotation or another refresh already advanced state.

Generations never wrap. The SDK supplies no refresh task, clock, expiry
decision, queue, retry, or token source.

## Secret Lifetime

Prefer `BearerToken::from_mut_bytes` or `from_secret_buffer`. Matching rotation
and refresh methods clear complete caller-owned mutable or guarded source
storage on success and rejection. `BearerToken::new(&str)` cannot clear its
immutable caller-owned source.

Adapter-owned active and retired token allocations clear through
`cloud-sdk-sanitization`. Retired tokens remain alive only while an in-flight
snapshot owns them. Authorization header bytes have their own cleanup-owning
allocation and clear after the final header clone drops.

These guarantees do not cover immutable caller copies, allocator internals,
reqwest, TLS, kernel/device buffers, swap, crash dumps, process abort,
deliberately leaked values, or remote systems. Rotate and revoke provider
tokens according to provider policy.

## Failure And Retry

Scope, endpoint, token-validation, stale-generation, and generation-exhaustion
errors are payload-free. Scope failure occurs before network work. This layer
does not retry. Later operation policy must combine authentication outcome with
delivery phase and idempotency before any retry decision.

## Verification

The release checks cover:

- every required, optional, forbidden, omitted, supplied, and mismatched field;
- unscoped bypass and HTTP/custom-endpoint confusion;
- scope validation before header and network activity;
- blocking and async parity;
- mutable and guarded source cleanup on success and rejection;
- in-flight snapshots and retired-token cleanup;
- sequential and concurrent stale refresh races;
- poisoned lock recovery and generation exhaustion;
- cleanup-owned header copies and redacted diagnostics.

Run `scripts/check_bearer_authentication.sh` for the focused contract.
