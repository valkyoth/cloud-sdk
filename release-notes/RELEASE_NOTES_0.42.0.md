# cloud-sdk 0.42.0 Release Notes

Status: release candidate; pentest and final retest passed. Local and GitHub
release checks remain required before tagging.

Release date: 2026-07-30

## Overview

v0.42 adds type-separated Basic authentication clients and bounded canonical
signing inputs without changing the default no_std graph. It also
source-locks a narrow credential-free Robot protocol fixture before the
provider-neutral API freeze.

## Basic Authentication

- Added bounded, redacted `BasicUsername` and `BasicPassword` types.
- Rejected empty values, controls, non-ASCII ambiguity, spaces or colons in
  usernames, and aggregate authorization overflow.
- Added mutable and guarded source ingestion with cleanup on success and
  rejection.
- Added cleanup ownership for `user:password`, Base64 output, stored
  authorization bytes, and reqwest header copies.
- Added type-separated blocking and async Basic clients.
- Applied exact required provider, service, endpoint, audience, account, and
  tenant policy before header or network work.
- Shared all request, TLS, origin, response-limit, metadata, and cleanup logic
  with the bearer executor.

## Signing Inputs

- Added v2 length-framed canonical request bytes; no v1 constructor remains.
- Bound provider, service, normalized scheme/tagged canonical
  host/port/base path, optional audience/account/tenant, key ID, digest
  algorithm, and signature algorithm.
- Made equivalent IPv6 spellings produce one canonical representation.
- Bound exact method, final target, ordered selected headers, an internally
  produced digest of the retained exact request body, nonce, and caller time.
- Required each body hasher to report its algorithm and rejected a mismatch
  with the signed context before hashing.
- Added caller-supplied hashing and signing traits.
- Added validated `SignedRequest` output that retains the exact signed request.
- Added transactional bounded output and complete digest, canonical-input, and
  signature cleanup on errors, invalid lengths, unwind, and drop.
- Added no clock, randomness, algorithm, key, filesystem, or key-store
  dependency.

## Robot Protocol Fixture

- Locked the current official Robot document and digest.
- Added one non-executing read request and one non-executing repeated-form
  mutation fixture.
- Added success, general error, invalid-input, authentication rejection,
  quota, maintenance, and empty-success shapes.
- Kept all fixture data credential-free and outside publishable crates.
- Made no Robot operation-coverage claim; the complete inventory remains
  scheduled for v0.74.

## Dependency

`base64-ng 1.3.9` is pinned exactly with default features disabled. It is
activated only by explicit reqwest transport features and remains absent from
default and std-only graphs.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.42.0` | canonical signing input contracts |
| `cloud-sdk-hetzner` | `0.32.3` | dependency-only core range update |
| `cloud-sdk-reqwest` | `0.29.0` | Basic credentials and authenticated clients |
| `cloud-sdk-sanitization` | `0.16.0` | unchanged; not published |
| `cloud-sdk-testkit` | `0.24.2` | dependency-only core range update |

## Documentation

- [`docs/AUTHENTICATION_POLICY.md`](../docs/AUTHENTICATION_POLICY.md)
- [`docs/SIGNING_INPUT_POLICY.md`](../docs/SIGNING_INPUT_POLICY.md)
- [`docs/ROBOT_WIRE_SOURCE_LOCK.md`](../docs/ROBOT_WIRE_SOURCE_LOCK.md)
- [`docs/MIGRATION_0.42.0.md`](../docs/MIGRATION_0.42.0.md)
- [`docs/PUBLIC_API_REVIEW_0.42.0.md`](../docs/PUBLIC_API_REVIEW_0.42.0.md)
- [`docs/DEPENDENCY_REVIEW_0.42.0.md`](../docs/DEPENDENCY_REVIEW_0.42.0.md)

## Pentest

The iterative v0.42 pentest reviewed Basic authorization ownership and
cleanup, canonical signing domain separation, exact request/body coupling,
algorithm binding, endpoint identity canonicalization, signer output
validation, transactional cleanup, and regression-test integrity. Findings
were remediated and regression-tested.

The final retest passed commit
`faeb5d01c0d7c9fac717beeb743f5ddf4170a680`. See the
[`v0.42.0` pentest report](../security/pentest/v0.42.0.md).

## Release Gate

```text
v0.42.0 pentest stop passed. Tag only after the clean local release gate and
GitHub checks pass on the final release commit.
```
