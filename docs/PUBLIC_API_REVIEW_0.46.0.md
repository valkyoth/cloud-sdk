# v0.46.0 Public API Review

Date: 2026-08-02

Scope: canonical request identity, local idempotency binding, replayability,
and bounded single-owner retry decisions.

## Added API

`BodyReplayability` and `PreparedRequest::body_replayability` make immutable
body replay an explicit prepared-request capability. Nonempty bodies fail
closed until a provider marks its immutable snapshot replayable.

`FingerprintScope`, `CanonicalFingerprint`, `FingerprintRef`,
`DigestAlgorithm`, `FingerprintHasher`, `FingerprintDigest`, and
`RetrySubject` define exact or collision-resistant request identities.
Construction verifies endpoint admission and includes all provider, operation,
endpoint, target, prepared header name/value/sensitivity, body, and optional
account-scope fields under one versioned binary domain. Private subject fields
prevent request policy and fingerprint identity from being mixed. Public
diagnostics redact bytes and caller-buffer guards clear complete storage on
drop.

`IdempotencyIntent` holds exclusive access to bounded caller entropy without
copying the byte array through movable values and clears that storage on drop.
`IdempotencyBinding` consumes one intent and binds it to one fingerprint. Both
remain non-cloneable so safe code cannot duplicate local operation identity.

`MaxAttempts`, `MonotonicInstant`, `MonotonicDuration`, `RetryPolicy`,
`RetryEvent`, `RetryDecision`, `RetryPermit`, `RetryStopReason`, and
`RetryController` provide one non-cloneable mutable retry owner. Decisions
consume hard attempts and requested-delay budgets, include delay in the hard
elapsed deadline, and return a one-use post-sleep execution permit.
`RetryExecutionError` separates permit rejection from redacted prepared
execution failure. Blocking and executor-neutral async permit methods execute
the exact replay directly without returning a reusable prepared request.

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

Canonical scratch and borrowed digest output clear on success, failure, and
drop. Reviewed `subtle` fixed-time primitives compare equal-length fingerprint
bytes and validate intent zero state; lengths and algorithm IDs are public.
Fingerprint, intent, and body diagnostics are redacted. Mutation retries
require provider eligibility, idempotent semantics, immutable body replay, a
fresh binding, exact wire match, complete prepared-policy equality, and budget
availability. Policy equality covers service endpoint policy, operation
metadata, response policy, authentication policy, raw response policy,
operation identity, body replayability, and request-header sensitivity.
Unknown delivery is consumed as possibly sent. Non-idempotent and destructive
Hetzner operations remain non-retryable.

Wall-clock quota values and monotonic retry values are distinct types.
Rollback and arithmetic overflow fail closed. A permit holds an exclusive
controller-state borrow through blocking or async execution, and its final
observation advances controller time. The API adds no implicit sleep, jitter,
entropy, clock, task, transport implementation, or provider header.
