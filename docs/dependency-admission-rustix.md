# Dependency Admission: rustix

## Decision

Admit exact `rustix 1.1.4` only as a Unix-target development dependency of
`cloud-sdk-hetzner`, with default features disabled and only `fs`, `process`,
and `std` enabled.

The dependency is used by the ignored Robot live-smoke integration test to:

- open credential files once with `RDONLY`, `CLOEXEC`, and `NOFOLLOW`;
- retain an owned descriptor for metadata validation and bounded reads; and
- compare file and private-parent ownership with the effective user ID.

It is not a normal dependency of any published crate, does not enter default or
`no_std` graphs, and is not compiled for the non-Unix live-harness fallback.

## Why It Is Needed

The standard library does not expose portable Unix `O_NOFOLLOW` constants or a
safe effective-user-ID function. Pre-open path metadata plus `File::open` leaves
a substitution race, hand-maintained OS constants are fragile, and direct
`libc` calls would require first-party unsafe code forbidden by workspace
policy. `rustix` provides the required operations through a maintained safe API.

## Reviewed Boundary

- Version: `1.1.4`
- Registry checksum:
  `b6fe4565b9518b83ef4f91bb47ce29620ca828bd32cb7e408f0062e9930ba190`
- License: `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`
- Declared MSRV: Rust `1.63`
- Direct features: `fs`, `process`, `std`
- New locked transitives: `errno 0.3.14` and `linux-raw-sys 0.12.1`

Cargo registry metadata and the resulting exact locked graph were reviewed on
2026-08-18. Cargo Deny, RustSec, complete-SBOM, platform, MSRV, package, and
dependency-freshness gates remain mandatory.

## Residual Risk

`rustix` necessarily wraps platform system calls and uses platform-specific
backends internally. The harness therefore validates behavior through symlink,
hard-link, permissions, private-parent, ownership-policy, descriptor-read, and
cross-target compile tests. The root-owned launcher remains Unix-specific, and
live credential loading fails closed on non-Unix systems.

Filesystem caches, same-user compromise, a compromised kernel, and credentials
already exposed before this boundary remain outside the SDK's guarantees.
