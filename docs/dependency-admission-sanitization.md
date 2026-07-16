# Sanitization Dependency Admission

Status: admitted only through `cloud-sdk-sanitization` with default features
disabled.

## Decision

| Crate | Version | Role | License | Default features |
| --- | --- | --- | --- | --- |
| `sanitization` | `1.2.4` | volatile caller-buffer cleanup | MIT OR Apache-2.0 | disabled |

The dependency is the first-party crate published from
<https://github.com/valkyoth/sanitization>. It is no_std by default and has no
runtime dependencies with default features disabled.

`cloud-sdk-sanitization` exposes a narrow `sanitize_bytes` function, a borrowed
`SecretBuffer` guard, and the reviewed opt-in allocation-backed
`sanitization::SecretString`.
Provider crates remain independent of the implementation by default, and the
default `cloud-sdk` and `cloud-sdk-hetzner` graphs remain unchanged. The
Hetzner `serde` feature enables the allocation-backed boundary so decoded
secret strings can be cleared when their owned storage is dropped.

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

No interoperability features are enabled. In particular, the optional
`zeroize`, `subtle`, memory-locking, derive, and platform features are absent
from the admitted graph. Allocation is enabled only through
`cloud-sdk-sanitization/alloc`; `std` remains a separate opt-in feature.

## Verification

`scripts/check_sanitization_boundary.sh` verifies the exact admitted version,
absence of optional interoperability dependencies, isolation from facade and
provider default graphs, the bounded allocation feature relationship, package
compilation, and guard behavior tests.
