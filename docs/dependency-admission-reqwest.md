# Reqwest Transport Dependency Admission

Status: admitted only through `cloud-sdk-reqwest/blocking-rustls` and
`cloud-sdk-reqwest/async-rustls`, with reqwest default features disabled.

## Decision

| Crate | Version | Role | Default features |
| --- | --- | --- | --- |
| `reqwest` | `0.13.4` | blocking/async HTTP client and URL/header types | disabled |
| `bytes` | `1.12.1` | sanitized owned async request-body handoff | disabled |
| `hyper` | `1.10.1` | transitive HTTP implementation | transitive |
| `tokio` | `1.52.3` | reqwest runtime; direct dev-only async test executor | transitive/disabled |
| `url` | `2.5.8` | authority-preserving endpoint parsing | transitive |
| `rustls` | `0.23.41` | TLS implementation | transitive |
| `rustls-platform-verifier` | `0.7.0` | platform trust-store verification | transitive |
| `aws-lc-rs` | `1.17.1` | rustls cryptographic provider | transitive |
| `cloud-sdk-sanitization` | `0.13.4` | adapter-owned secret-buffer cleanup | disabled |
| `sanitization` | `1.2.4` | reviewed volatile cleanup primitive | disabled |

The exact complete graph is pinned by `Cargo.lock`, checked by `cargo deny`,
and recorded in the generated SBOM. All admitted licenses satisfy
`deny.toml`. The rustls trust-root data requires `CDLA-Permissive-2.0`, which
is explicitly admitted. No duplicate-version exception is active for the
transport graph; the boundary rejects legacy `windows-sys` `0.52.0` if it
becomes reachable again.

Aws-lc-sys introduces the workspace's first native dependency build script. It
invokes a C/CMake toolchain to compile vendored C and assembly cryptographic
code. Cargo authenticates the published crate archive against the checksum
pinned in `Cargo.lock`; this does not remove the trust placed in the crate's
build script, bundled source, compiler, assembler, CMake, linker, or build
host. Release CI and reproducible/offline builders must use pinned, audited
build images and toolchains. Offline preparation must preserve Cargo's
authenticated package checksum rather than copying unauthenticated source
trees. The v0.24 dependency-hardening pass must revisit this native surface.

The version review used the reqwest 0.13.4 crate metadata, feature list, API
documentation, and upstream source:

- <https://crates.io/crates/reqwest/0.13.4>
- <https://docs.rs/reqwest/0.13.4/reqwest/>
- <https://github.com/seanmonstar/reqwest/tree/v0.13.4>

## Feature Boundary

The crate remains no_std and transport-free by default. Its `std` feature also
does not admit reqwest. `blocking-rustls` enables:

- `std`;
- `cloud-sdk-sanitization`;
- `reqwest/blocking`;
- `reqwest/rustls`.

`async-rustls` enables:

- `std`;
- `bytes`;
- `cloud-sdk-sanitization`;
- `reqwest/rustls`.

It deliberately does not enable `reqwest/blocking`. The core async contract
and testkit do not depend on Tokio. The concrete async reqwest adapter requires
callers to poll requests from an active Tokio executor; it does not create,
install, or own a runtime.

Reqwest default features are disabled. Native TLS, cookies, JSON, multipart,
SOCKS, system proxy discovery, redirects, retries, referer generation, and
response decompression are not admitted. HTTP/2 and Hickory DNS are also absent
from both production feature graphs. A
separate locked, non-published test fixture deliberately enables both on the
same reqwest instance and builds both adapters to exercise Cargo feature
unification against the runtime overrides. Its local `cloud-sdk-reqwest`
dependency is pinned exactly to `0.15.0`.

The fixture lockfile is a separate 200-package tooling graph. Release and CI
gates apply the root advisory, license, and source policy to that lockfile,
audit it independently, and generate a dedicated SPDX SBOM. The root
production graph retains `multiple-versions = "deny"`. The fixture does not
apply that ban because enabling Hickory and rustls platform verification
together currently requires `core-foundation` `0.9.4` and `0.10.1` on Apple
targets. This exception is confined to the non-published adversarial fixture;
the production reqwest feature graph excludes Hickory.

## Client Policy

Production builders enforce:

- HTTPS-only endpoints;
- rustls with TLS 1.2 as the minimum protocol version;
- platform trust-store certificate verification;
- no invalid-certificate or invalid-hostname bypass;
- HTTP/1 only, even if another dependency enables reqwest HTTP/2;
- the system resolver, even if another dependency enables reqwest Hickory DNS;
- no redirects and no automatic retries;
- no proxy use or environment proxy discovery;
- no gzip, Brotli, Zstandard, or deflate response decompression;
- explicit nonzero total and connect timeouts, each at most 300 seconds;
- explicit validated user agent and bearer authorization;
- caller-sized bounded response bodies;
- strict all-or-none decimal parsing of the three provider-neutral rate-limit
  response metadata fields;
- async response accumulation limited to caller capacity, followed by one
  complete-success copy into the caller buffer.

The adapter validates the final scheme, host, and port against the configured
endpoint. Request targets are origin-form, and encoded path separators,
backslashes, dot bytes, controls, non-ASCII bytes, fragments, and malformed
percent escapes are rejected before credentials are attached.

Platform trust roots intentionally follow host policy. A compromised or
attacker-extended host trust store, including an enterprise interception root,
can therefore validate a hostile endpoint. The v0.24 dependency-hardening
milestone will evaluate a separately reviewed deterministic root-store option;
v0.17 does not claim certificate or public-key pinning.

A separate `blocking-rustls-fips` transport is assigned to v0.23.0. It must
use and verify the rustls FIPS provider and configuration explicitly; callers
enabling unrelated dependency features is not an SDK FIPS guarantee.

## Secret Boundary

`BearerToken` copies the complete authorization value into adapter-owned
storage, marks reqwest's header value sensitive, redacts diagnostics, and
volatile-clears its owned bytes on drop through `cloud-sdk-sanitization`.
Request-body copies and caller response buffers are also cleared on their
owned failure or drop paths. Async response data is accumulated in a
pre-reserved adapter-owned sanitized buffer and copied into the already-cleared
caller buffer only after complete success. Dropping the future during connect,
send, or response reading drops and clears that adapter-owned accumulation.

The adapter cannot clear the caller's original immutable token string,
reqwest/header/TLS copies, operating-system socket buffers, kernel memory,
swap, crash dumps, or remote systems. Callers must generate tokens with an
appropriate CSPRNG, rotate and revoke them operationally, minimize scope, and
clear any caller-owned mutable secret buffers after use.

The rustls dependency graph internally contains `zeroize` through rustls,
aws-lc-rs, and rustls-pki-types. Workspace crates neither depend on it directly
nor use it as their cleanup API; all workspace-owned cleanup remains behind
`cloud-sdk-sanitization`. Every first-party manifest is checked to prevent a
direct `zeroize` admission.

## Platform Scope

The core and provider crates remain portable no_std libraries. The optional
adapters require a reqwest/rustls-supported std target, platform trust roots,
networking, threads, and the native build requirements of aws-lc-sys. The async
adapter additionally requires an active Tokio executor supplied by the caller.
Linux, Windows, macOS, BSD, Android, and iOS remain intended targets, but a
platform is not claimed as release-verified until its target check or CI job is
recorded. Aesynx compatibility remains a future no_std integration concern and
does not enable this std adapter.

## Verification

`scripts/check_reqwest_boundary.sh` verifies the exact top-level versions,
default and std graph isolation, separate blocking/async required and forbidden
reqwest features,
absence of native TLS and decompression dependencies, direct-zeroize
exclusion, hardened builder policy, adversarial HTTP/2/Hickory feature
unification, focused tests, fixture lockfile policy and audit coverage, and
package verification.
The v0.17 release gate additionally runs the full workspace checks, MSRV
matrix, cargo-deny, cargo-audit, upstream API drift checks, and pentest evidence
validation.
