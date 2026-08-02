# Migrating To v0.46

v0.46 adds source-locked retry and local idempotency policy without adding a
runtime, clock, random source, allocator, network client, or cryptographic
implementation to the default graph.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.46.0"
cloud-sdk-hetzner = "0.36.0"
cloud-sdk-reqwest = { version = "0.31.1", features = ["blocking-rustls"] }
cloud-sdk-sanitization = "0.16.0"
cloud-sdk-testkit = "0.26.0"
```

`cloud-sdk-sanitization` is unchanged and is not published. Reqwest changes
only because its core dependency advances; it does not gain transport-owned
retry behavior.

## Prepared Body Capability

`PreparedRequest::body_replayability` is new. Bodyless core requests are
replayable. A nonempty request defaults to `NotReplayable` until a provider
whose preparation owns an immutable byte snapshot calls
`with_replayable_body`. Hetzner preparation now makes that guarantee for every
active operation. `PreparedRequestRecord` exposes the capability to testkit
assertions.

## Retry Migration

Do not loop around `execute_blocking`, `execute_async`, or a raw transport call
using method-based assumptions. Instead:

1. Build an exact canonical fingerprint or reviewed collision-resistant
   digest for the prepared request and admitted endpoint. Keep exact or digest
   output in its caller-owned cleanup guard.
2. For an eligible mutation, obtain fresh mutable CSPRNG bytes and bind one
   `IdempotencyIntent` to that fingerprint. A valid intent borrows one source
   buffer until drop and then clears it; invalid input clears immediately.
3. Create one `RetryController` with nonzero attempts and hard cumulative-delay
   and monotonic-elapsed budgets from `fingerprint.subject()`. Request policy
   and fingerprint identity can no longer be supplied independently. The
   controller retains the complete initial prepared policy identity.
4. Execute the initial attempt once.
5. Feed conservative delivery phase or response status, the rebuilt exact
   subject, caller-selected delay/jitter, and monotonic time into
   `decide_retry`.
6. For `RetryDecision::Retry(permit)`, sleep outside the SDK and consume
   `permit.execute_blocking(...)` or `permit.execute_async(...)`. The permit
   rechecks time and executes its exact request without returning a reusable
   `PreparedRequest`.

The `v2` fingerprint binds each request header's sensitivity marker as well as
its name and value. The controller also compares those markers independently
as prepared policy. It rejects wire fingerprint mismatch, any difference in
complete prepared policies, unadmitted endpoints, monotonic rollback, delay
overflow, and a mutation binding created for another request. It stops on
ineligible metadata, non-replayable bodies, non-transient responses, projected
deadline overrun, or exhausted budgets. A permit exclusively borrows
controller clock state until execution, preventing simultaneously outstanding
attempts in safe code. Its post-sleep observation advances the same monotonic
state used by the next decision.

The idempotency binding is local replay policy. It does not add a provider
header and cannot make a source-locked non-idempotent or destructive Hetzner
operation retryable.

See [`RETRY_AND_IDEMPOTENCY.md`](RETRY_AND_IDEMPOTENCY.md) for the complete
format, policy table, ownership rules, and security boundaries.
