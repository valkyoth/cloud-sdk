# cloud-sdk 0.29.0 Release Notes

Status: implementation stop reached; pentest required before tagging.

Release date: pending

## Overview

`0.29.0` adds one provider-neutral, allocation-free contract for preparing a
validated operation and checking its response. A prepared request binds method,
target, body, endpoint identity, operation safety metadata, retry eligibility,
cost intent, accepted statuses and media types, body policy, and response limit.

The default workspace remains `no_std`, transport-free, runtime-free, and free
of new third-party dependencies.

## Prepared Request Contract

- `PrepareOperation` consumes explicit caller-owned target/body storage and
  returns one complete `PreparedRequest`.
- `OperationMetadata` distinguishes read-only, mutation, and destructive
  impact from safe, idempotent, and non-idempotent semantics.
- Retry eligibility and cost-bearing intent are explicit independent fields;
  there is no permissive default.
- Metadata construction rejects safe state changes, non-safe reads, and retry
  eligibility for non-idempotent operations.
- `ProviderService` binds provider family to immutable expected endpoint
  identity before credentials can be used.
- Blocking and async prepared execution send exactly once and own no retry,
  delay, clock, task, queue, or executor behavior.

## Checked Response Policy

- Execution lends only the smaller of caller response storage and the
  operation's admitted maximum to the transport.
- `ResponsePolicy` checks expected success status, initialized body length,
  required/optional/forbidden body shape, and required/optional/forbidden media
  type before producing `CheckedResponse`.
- `TransportResponse` carries an optional bounded owned response content type.
- Content-type parsing validates the media essence and conservative ASCII
  parameter syntax without allocation.
- Blocking and async reqwest adapters reject duplicate, non-textual, oversized,
  or malformed response content types before returning body bytes.

## Testkit

- Mocks can be bound to immutable endpoint identity for prepared execution.
- Expected requests can match exact request content type.
- Response fixtures model missing, accepted, unexpected, malformed, and
  duplicate-policy content-type cases.
- `PreparedRequestRecord` captures redacted request shape and complete service,
  operation, retry, cost, and response policy metadata.
- Blocking and async conformance tests cover endpoint mismatch, unexpected
  status, missing or unexpected media type, empty body, oversized response, and
  retry-classification assertions.

## Independent Crate Versions

- `cloud-sdk` publishes code release `0.29.0`.
- `cloud-sdk-hetzner` publishes dependency-only patch `0.22.1`.
- `cloud-sdk-reqwest` publishes code release `0.20.0`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.15`.
- `cloud-sdk-testkit` publishes code release `0.18.0`.

No retired provider-specific helper crate is present in the publish plan.

## Migration

See [`docs/MIGRATION_0.29.0.md`](../docs/MIGRATION_0.29.0.md) for prepared
operation, response content-type, reqwest, and testkit changes.

## Verification

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-targets --all-features`
- `scripts/checks.sh`
- `scripts/release_crates.py --check`
- `scripts/test-release-crates.py`
- `scripts/validate-file-lengths.sh`
- `scripts/check_sbom_freshness.sh`
- `scripts/release_0_29_gate.sh`
- `scripts/release_crates.py --dry-run --yes --version 0.29.0`

## Stop Gate

```text
v0.29.0 implementation stop reached. Run pentest for this exact commit.
```
