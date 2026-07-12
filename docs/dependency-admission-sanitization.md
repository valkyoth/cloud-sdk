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

`cloud-sdk-sanitization` exposes a narrow `sanitize_bytes` function and a
`SecretBuffer` guard. Provider crates remain independent of the implementation,
and the default `cloud-sdk` and `cloud-sdk-hetzner` graphs remain unchanged.

## Security Boundary

The admitted primitive uses volatile byte writes through the dependency's
reviewed internal unsafe boundary. This workspace keeps `unsafe_code =
"forbid"` for its own crates and does not duplicate that implementation.

The guard clears its full borrowed destination on drop, including after errors,
early returns, and unwind where unwind exists. It cannot clear immutable source
strings, transport copies, kernel buffers, crash dumps, swap, remote systems,
or copies outside the guarded slice.

No interoperability features are enabled. In particular, the optional
`zeroize`, `subtle`, allocation, memory-locking, derive, and platform features
are absent from the admitted graph.

## Verification

`scripts/check_sanitization_boundary.sh` verifies the exact admitted version,
absence of optional interoperability dependencies, isolation from facade and
provider default graphs, package compilation, and guard behavior tests.
