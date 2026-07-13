# cloud-sdk 0.19.0 Release Notes

Status: implementation stop reached; pentest and retest required before tagging.

## Overview

`0.19.0` adds the first opt-in live validation surface for the existing
Hetzner provider and hardened blocking transport. The harness exercises only
read-only catalog operations and remains ignored during ordinary test runs.
No network, TLS, filesystem, allocator, or runtime dependency enters a default
SDK or provider graph.

## Read-Only Harness

- Probes locations, server types, load balancer types, ISOs, public system
  images, and pricing against the fixed official Cloud API v1 origin.
- Builds paginated targets through typed `cloud-sdk-hetzner` catalog domains
  and sends them through `cloud-sdk-reqwest/blocking-rustls`.
- Requires HTTP 200, the expected resource envelope shape, and strict page-one
  metadata with the requested one-entry page bound.
- Bounds each response to 1 MiB and never prints a response body or resource
  identifier.
- Uses explicit 10-second connect and 30-second total timeouts with redirects,
  retries, proxies, referers, decompression, HTTP/2, and Hickory DNS disabled by
  the existing transport policy.

## Credential Boundary

- Requires both the ignored-test opt-in and
  `CLOUD_SDK_HETZNER_LIVE_MODE=read-only`; the wrapper sets the latter only for
  `--read-only` execution.
- Accepts only `CLOUD_SDK_HETZNER_TOKEN_FILE`, never a raw token environment
  variable or command-line token.
- Rejects empty paths, symlinks, non-regular files, files above the bounded
  token size, changed Unix device/inode identity during open, and Unix group or
  world permission bits.
- Accepts an exact token with at most one LF or CRLF terminator and rejects
  boundary whitespace.
- Preallocates the complete bounded token-read capacity before file I/O so
  buffer growth cannot retire allocations containing plaintext token fragments.
- Clears bounded caller-owned token and response source buffers through
  `cloud-sdk-sanitization` on success and failure. Adapter, TLS, OS, shell, and
  filesystem copies remain documented operational boundaries.

## Destructive Testing

Mutation execution is intentionally absent from `0.19.0`. The documented plan
requires a dedicated disposable project, a short-lived read-write token, a
unique `cloud-sdk-live-...` resource prefix, explicit operation and pricing
review, inventory before and after the run, cleanup on every path, and manual
verification that no prefixed resources remain. A later implementation must
use a separate command and cannot reuse the read-only opt-in.

## Version Plan

- `cloud-sdk` publishes metadata release `0.19.0`.
- `cloud-sdk-hetzner` publishes code release `0.17.0`.
- `cloud-sdk-reqwest` publishes dependency-only patch `0.15.1`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.5`.
- `cloud-sdk-testkit` publishes dependency-only patch `0.15.1`.
- Retired and provider-scoped helper packages remain rejected and unpublished.
- Publisher validation requires `cloud-sdk.previous_version` to match the
  latest semantic release tag earlier than the planned release.

## Verification

- `scripts/checks.sh`
- `scripts/smoke_hetzner_live.sh --check`
- `cargo test -p cloud-sdk-hetzner --test live_smoke --all-features`
- `scripts/check_reqwest_boundary.sh`
- `scripts/check_rust_version_matrix.sh`
- `scripts/release_0_19_gate.sh`
- `cargo deny check`
- `cargo audit`
- `git diff --check`

The authenticated `--read-only` run is manual and requires a caller-provided
least-privilege token. It is not executed by default CI or release automation.
The release gate also rejects an uncommitted worktree before binding its checks
to the reviewed `HEAD`.

## Pentest

Pentest and retest are required before tagging. The final report must be the
only change in the direct child of the reviewed implementation commit.
