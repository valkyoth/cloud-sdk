# v0.46.0 Public API Review

Date: 2026-08-02

Scope: canonical request identity, local idempotency binding, replayability,
and bounded single-owner retry decisions.

## Added API

`BodyReplayability` and `PreparedRequest::body_replayability` make immutable
body replay an explicit prepared-request capability. Nonempty bodies fail
closed until a provider marks its immutable snapshot replayable.

`FingerprintScope`, `CanonicalFingerprint`, `FingerprintRef`,
`DigestAlgorithm`, `FingerprintHasher`, and `FingerprintDigest` define exact
or collision-resistant request identities. Construction includes all provider,
operation, endpoint, target, prepared header, body, and optional account-scope
fields under one versioned binary domain. Public diagnostics redact bytes and
cleanup-owning values clear complete storage on drop.

`IdempotencyIntent` moves bounded caller entropy into fixed cleanup-owning
storage without exposing it and clears the mutable source on every path.
`IdempotencyBinding` consumes one intent and binds it to one fingerprint. Both
remain non-cloneable so safe code cannot duplicate local operation identity.

`MaxAttempts`, `MonotonicInstant`, `MonotonicDuration`, `RetryPolicy`,
`RetryEvent`, `RetryDecision`, `RetryStopReason`, and `RetryController` provide
one non-cloneable mutable retry owner. Decisions consume hard attempts and
requested-delay budgets and validate monotonic elapsed time.

`PreparedRequestRecord::body_replayability` adds redacted testkit evidence.

## Changed API

Hetzner prepared requests now call `with_replayable_body` after completing
transactional target/body preparation. No endpoint request or response shape
changes. Reqwest advances by dependency-only patch and continues to issue one
attempt per transport call.

## Security Review

The fingerprint format has explicit version, domain, tags, and lengths. Exact
comparison and admitted digest comparison avoid Rust's non-cryptographic
`Hash`. Digest algorithms have fixed collision-resistant identities and exact
lengths; caller implementations remain reviewed trust boundaries.

Canonical scratch and digest output clear on success, failure, and drop.
Fingerprint, intent, and body diagnostics are redacted. Mutation retries
require provider eligibility, idempotent semantics, immutable body replay, a
fresh binding, exact request match, and budget availability. Unknown delivery
is consumed as possibly sent. Non-idempotent and destructive Hetzner
operations remain non-retryable.

Wall-clock quota values and monotonic retry values are distinct types.
Rollback and arithmetic overflow fail closed. The API adds no implicit sleep,
jitter, entropy, clock, task, transport execution, or provider header.
