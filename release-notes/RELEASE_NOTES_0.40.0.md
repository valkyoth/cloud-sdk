# cloud-sdk 0.40.0 Release Notes

Status: release candidate; pentest and final retest passed. Local and GitHub
release checks remain required before tagging.

Release date: 2026-07-29

## Overview

v0.40 adds the credential-free raw HTTP execution layer. Complete validated
requests now execute through bounded caller response storage with conservative
delivery phases, explicit response-wire policy, and no implicit provider
authentication or retry behavior.

## Raw HTTP Contract

- Added blocking and executor-neutral async raw executor traits.
- Added separate success/error body limits and media policies.
- Added bounded informational-response tracking and rejected HTTP 101.
- Canceled in-flight requests immediately when informational-response policy
  is exceeded, without waiting for a final head or total timeout.
- Defined HEAD, 204, 304, duplicate-header, media, and trailer behavior.
- Dropped all unadmitted response headers and forbade credential, cookie,
  framing, proxy, and upgrade metadata admission.
- Added payload-redacting `NotSent`, `PossiblySent`, and `ResponseStarted`
  failures, with unknown state mapped to `PossiblySent`.
- Added core cleanup-owning response attempts so failure, timeout, unwind, and
  async cancellation cannot contaminate later writer reuse.
- Made body/header mutation and commitment available only through
  `ResponseAttempt`; compile-fail coverage prevents external transport bypass.
- Defined `ResponseStarted` as any observed informational or final head.

## Reqwest Adapter

- Added credential-free blocking and async raw clients.
- Shared one Hyper HTTP/1 engine across ordinary, deterministic-root, and FIPS
  modes.
- Bounded response heads to 100 fields, 64 KiB of encoded field bytes, and a
  64 KiB pinned Hyper parser buffer.
- Counted actual response data frames and bytes directly into caller storage.
- Rejected declared and observed trailers.
- Disabled idle pooling and automatic canceled-request retries.
- Staged request body and header values in cleanup-owning allocations.
- Rejected raw request bodies above 8 MiB before first-party reqwest
  adapter-owned allocation and documented that scope separately from raw
  traits.
- Retained explicit total/connect deadlines and TLS trust policy.

## Testkit

- Added deterministic raw executor faults for every delivery phase.
- Added blocking and async interim-response tests.
- Added isolated fuzz targets for post-parse response validation/body
  accounting and the actual in-memory Hyper HTTP/1 wire state machine.
- Gave wire fuzzing a 66,560-byte target-specific input cap, canonical CRLF
  parser-state seeds, and below/exact/plus-one encoded-head regressions.
- Added 101, trailer, duplicate, missing-length overflow, HEAD, 204, media,
  unknown-header, cookie, and authentication-confusion fixtures.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.40.0` | raw executor and response-wire policy |
| `cloud-sdk-hetzner` | `0.32.1` | dependency-only core range update |
| `cloud-sdk-reqwest` | `0.27.0` | raw blocking/async/TLS executors |
| `cloud-sdk-sanitization` | `0.16.0` | unchanged; not published |
| `cloud-sdk-testkit` | `0.24.0` | delivery-phase fault injection |

## Documentation

- [`docs/RAW_HTTP_EXECUTOR.md`](../docs/RAW_HTTP_EXECUTOR.md)
- [`docs/MIGRATION_0.40.0.md`](../docs/MIGRATION_0.40.0.md)
- [`docs/PUBLIC_API_REVIEW_0.40.0.md`](../docs/PUBLIC_API_REVIEW_0.40.0.md)
- [`docs/DEPENDENCY_REVIEW_0.40.0.md`](../docs/DEPENDENCY_REVIEW_0.40.0.md)

## Pentest

The v0.40 pentest findings covered prompt informational-response rejection,
production wire-parser fuzzing, transactional response cleanup, request-copy
allocation bounds, exact wire-limit coverage, and precise delivery and
dependency documentation. All findings were remediated and regression-tested.

The final retest passed commit
`ba0a60f2e9377300f7761f9429a73e0e25124213`. See the
[`v0.40.0` pentest report](../security/pentest/v0.40.0.md).

## Release Gate

```text
v0.40.0 pentest stop passed. Tag only after the clean local release gate and
GitHub checks pass on the final release commit.
```
