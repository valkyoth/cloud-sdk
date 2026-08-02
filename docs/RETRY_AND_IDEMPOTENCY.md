# Retry And Idempotency Policy

`cloud-sdk` keeps retry execution explicit and provider-neutral. The core
contract classifies one prepared request, binds its exact wire identity, and
applies hard attempt, requested-delay, and monotonic elapsed budgets. It does
not acquire clocks or entropy, sleep, add jitter, schedule work, send an
idempotency header, or execute transport.

## Provider Policy Table

Hetzner preparation source-locks every active operation into one class:

| Operation class | Semantics | Retry metadata | Additional v0.46 rule |
| --- | --- | --- | --- |
| Read only | Safe | Explicit caller policy | Exact replayable request may retry. |
| Idempotent mutation | Idempotent | Explicit caller policy | Requires a fresh fingerprint-bound intent. |
| Non-idempotent mutation | Non-idempotent | Never | No automatic retry. |
| Idempotent destructive | Idempotent | Never | No automatic retry. |
| Non-idempotent destructive | Non-idempotent | Never | No automatic retry. |

These classifications come from provider endpoint metadata, not from the HTTP
method. Every Hetzner prepared request is an immutable target/body snapshot and
is marked byte-for-byte replayable. A custom provider must mark replayability
only when it can make the same guarantee.

The local idempotency binding does not imply provider-side deduplication.
Hetzner operations classified as non-idempotent or destructive remain
non-retryable even when a caller has an intent identifier.

## Canonical Fingerprint

`build_canonical_fingerprint` writes a versioned, domain-separated binary
format into caller storage. Length-prefixed fields bind:

- provider, service, and operation identifiers;
- method, endpoint scheme, canonical DNS/IPv4/IPv6 identity, effective port,
  and base path;
- exact path, query presence, and exact query bytes;
- every prepared request header name and value;
- exact request body bytes; and
- explicit account or tenant scope, including absence.

The domain is `cloud-sdk/retry-fingerprint/v1`. Tags and big-endian lengths
separate every field. Header names are lowercased because HTTP names are
case-insensitive; values and ordering remain exact. Adapter-owned
authentication is excluded because credentials may rotate without changing
the intended provider operation.

Canonical bytes are redacted, cannot be extracted through the public API, and
the complete caller buffer is volatile-cleared when its guard drops. Use an
exact fingerprint reference while the guard lives, or call
`build_fingerprint_digest` with a reviewed `FingerprintHasher` implementation.

The admitted digest algorithm identifiers are SHA-256, SHA-384, SHA-512, and
BLAKE3 with exact output lengths. The SDK rejects unknown algorithm identifiers
and incorrect output lengths. The caller implementation is a security trust
boundary: it must compute the algorithm it declares over exactly the supplied
canonical bytes. Rust `Hash`, CRC, truncated output, and other
non-collision-resistant substitutes violate the contract.

## Fresh Intent Binding

For every intentional mutation, obtain at least 16 fresh mutable bytes from a
CSPRNG and construct `IdempotencyIntent`. Construction moves the bytes into
fixed owned storage and clears the complete source buffer on success or
failure. The owned copy clears on drop. The type rejects short, oversized, and
all-zero values and is neither `Copy` nor `Clone`. It can validate shape but
cannot prove entropy quality or global uniqueness.

Move the intent into `IdempotencyBinding::bind(intent, fingerprint)`. This
creates one local operation identity from fresh intent plus exact request
identity. Two intentionally separate identical operations must use different
fresh intents. The controller rejects a binding made for another fingerprint.
The binding is local policy evidence and is not automatically transmitted.

## Single Retry Owner

Create one `RetryController` from the prepared request, its fingerprint, an
optional binding, complete budgets, and a caller-observed monotonic start:

```rust,ignore
use cloud_sdk::retry::{
    IdempotencyBinding, IdempotencyIntent, MaxAttempts, MonotonicDuration,
    MonotonicInstant, RetryController, RetryPolicy,
};

let policy = RetryPolicy::new(
    MaxAttempts::new(3)?,
    MonotonicDuration::new(30_000),
    MonotonicDuration::new(120_000),
);
let intent = IdempotencyIntent::new(&mut csprng_bytes)?;
let binding = IdempotencyBinding::bind(intent, fingerprint);
let mut retries = RetryController::new(
    prepared,
    fingerprint,
    Some(binding),
    policy,
    MonotonicInstant::new(started_ticks),
)?;
```

`RetryController` is neither `Copy` nor `Clone`, and decisions require
`&mut self`. This makes one value the attempt and delay-budget owner and
prevents safe fan-out from that owner. Transports and adapters must continue to
perform exactly one attempt per call; do not enable a second transport-owned
retry layer.

After the initial attempt, pass one conservative event, a newly rebuilt
fingerprint, caller-selected backoff plus jitter, and a monotonic observation
to `decide_retry`. A returned `RetryDecision::Retry` consumes the next attempt
and charges the complete requested delay. A stop decision does not execute or
sleep.

Only `429` and `5xx` responses are transient at this neutral layer. Provider
error decoding may impose stricter policy before calling the controller.
Unknown transport delivery must already be represented as `PossiblySent`.
Possibly-sent mutations retry only when provider metadata says the operation
is idempotent and the controller owns a matching fresh binding.

## Time And Budget Rules

`WallClockTimestamp` remains for HTTP-date and quota interpretation.
`MonotonicInstant` and `MonotonicDuration` are separate retry-budget types.
Their tick unit is caller-defined but must remain consistent for one
controller.

- `MaxAttempts` is nonzero and includes the initial attempt.
- Maximum cumulative delay charges requested backoff and jitter before retry.
- Maximum elapsed time uses caller monotonic observations.
- Arithmetic overflow fails closed.
- A backward monotonic observation is an error and never extends a budget.
- Zero delay is data, not permission to busy-loop beyond the attempt bound.

Cancellation, deadlines, sleeping, randomness, and concurrency limits remain
application responsibilities. Never hold credentials or secret payloads in
ordinary diagnostic values. Keep canonical scratch and request preparation
buffers under their cleanup-owning guards until transport use is complete.
