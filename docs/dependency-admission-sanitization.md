# Sanitization Dependency Admission

Status: admitted only through the mandatory `cloud-sdk-sanitization` boundary
with default features disabled.

Checked: 2026-08-03.

## Decision

| Crate | Version | Role | License | Default features |
| --- | --- | --- | --- | --- |
| `sanitization` | `2.0.3` | volatile caller-buffer and owned-secret cleanup | MIT OR Apache-2.0 | disabled |

The dependency is the first-party crate published from
<https://github.com/valkyoth/sanitization>. Version `2.0.3` retains Rust 1.90
and `no_std`, and has no runtime dependencies with default features disabled.
The upstream default `asm-compare` feature is deliberately not admitted.

`cloud-sdk-sanitization` exposes narrow `sanitize_bytes` and `sanitize_value`
functions, a borrowed `SecretBuffer` guard, the reviewed opt-in
allocation-backed `sanitization::SecretString`, and bounded fallible protected
string growth.
Since v0.38, the provider-neutral core depends on this boundary so response
cleanup cannot be delegated to an untrusted transport implementation. The
default `cloud-sdk` and provider graphs therefore contain only the two
first-party boundary crates plus `sanitization`; they remain allocation-free
and `no_std`. The Hetzner `serde` feature additionally enables alloc-backed
secret strings.

## Security Boundary

The admitted primitive uses volatile byte writes through the dependency's
reviewed internal unsafe boundary. This workspace keeps `unsafe_code =
"forbid"` for its own crates and does not duplicate that implementation.

The borrowed guard clears its full destination on drop, including after errors,
early returns, and unwind where unwind exists. `SecretString` consumes a
`String` without making another plaintext copy, clears its full allocation
capacity on drop, clears old allocations before growth, and exposes UTF-8 only
through checked closures. Neither guard can clear immutable source strings,
transport copies, kernel buffers, crash dumps, swap, remote systems, allocator
metadata, or copies outside guarded storage.

The facade's fallible append prepares bounded replacement storage before
copying, clears the old protected allocation before swapping, and reports
length, capacity, allocation, or UTF-8-invariant failure without payload data.

No interoperability or native hardening features are enabled. In particular,
the optional `zeroize-interop`, `subtle-interop`, memory locking, guard pages,
cache flushing, canaries, register scrubbing, derive, Serde, hardware secret,
split-secret, and multi-pass features are absent from the admitted graph.
Allocation is enabled only through
`cloud-sdk-sanitization/alloc`; `std` remains a separate opt-in feature.

The `1.2.5` to `2.0.3` major update was reviewed against the exact APIs used by
this workspace. Upstream `sanitize_bytes` became `wipe::bytes`; the
`cloud-sdk-sanitization::sanitize_bytes` wrapper preserves the existing public
API. `SecretString::from_string`, closure-scoped UTF-8 access, capacity-aware
growth cleanup, and drop cleanup remain available. No newly added default or
optional feature is enabled through this boundary.
The release-specific evidence and Cargo checksum are recorded in
[`DEPENDENCY_REVIEW_0.32.0.md`](DEPENDENCY_REVIEW_0.32.0.md).

## Verification

`scripts/check_sanitization_boundary.sh` verifies the exact admitted version,
the one-way `cloud-sdk-sanitization -> sanitization` dependency, mandatory
core/provider inclusion, absence of optional interoperability dependencies,
the bounded allocation feature relationship, package compilation, and guard
behavior tests. `scripts/check_response_cleanup.sh` rejects ordinary
first-party zero fills and exercises the complete response lifecycle.
