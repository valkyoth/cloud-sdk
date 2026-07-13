# Reqwest Blocking Transport Dependency Admission

Status: admitted only through `cloud-sdk-reqwest/blocking-rustls` with reqwest
default features disabled.

## Decision

| Crate | Version | Role | Default features |
| --- | --- | --- | --- |
| `reqwest` | `0.13.4` | blocking HTTP client and URL/header types | disabled |
| `hyper` | `1.10.1` | transitive HTTP implementation | transitive |
| `tokio` | `1.52.3` | transitive blocking-client runtime support | transitive |
| `url` | `2.5.8` | authority-preserving endpoint parsing | transitive |
| `rustls` | `0.23.41` | TLS implementation | transitive |
| `rustls-platform-verifier` | `0.7.0` | platform trust-store verification | transitive |
| `aws-lc-rs` | `1.17.1` | rustls cryptographic provider | transitive |
| `cloud-sdk-sanitization` | `0.13.2` | adapter-owned secret-buffer cleanup | disabled |
| `sanitization` | `1.2.4` | reviewed volatile cleanup primitive | disabled |

The exact complete graph is pinned by `Cargo.lock`, checked by `cargo deny`,
and recorded in the generated SBOM. All admitted licenses satisfy
`deny.toml`. The rustls trust-root data requires `CDLA-Permissive-2.0`, which
is explicitly admitted. `windows-sys` `0.52.0` is a narrow duplicate exception
because rustls platform verification has not yet converged on the `0.61` line
used by Tokio and rustls-native-certs.

The version review used the reqwest 0.13.4 crate metadata, feature list, API
documentation, and upstream source:

- <https://crates.io/crates/reqwest/0.13.4>
- <https://docs.rs/reqwest/0.13.4/reqwest/>
- <https://github.com/seanmonstar/reqwest/tree/v0.13.4>

## Feature Boundary

The crate remains no_std and transport-free by default. Its `std` feature also
does not admit reqwest. Only `blocking-rustls` enables:

- `std`;
- `cloud-sdk-sanitization`;
- `reqwest/blocking`;
- `reqwest/rustls`.

Reqwest default features are disabled. Native TLS, cookies, JSON, multipart,
SOCKS, system proxy discovery, redirects, retries, referer generation, and
response decompression are not admitted. Async APIs are not exposed by this
release, although reqwest's blocking implementation internally uses Tokio.

## Client Policy

Production builders enforce:

- HTTPS-only endpoints;
- rustls with TLS 1.2 as the minimum protocol version;
- platform trust-store certificate verification;
- no invalid-certificate or invalid-hostname bypass;
- no redirects and no automatic retries;
- no proxy use or environment proxy discovery;
- no gzip, Brotli, Zstandard, or deflate response decompression;
- explicit nonzero total and connect timeouts, each at most 300 seconds;
- explicit validated user agent and bearer authorization;
- caller-sized bounded response bodies.

The adapter validates the final scheme, host, and port against the configured
endpoint. Request targets are origin-form, and encoded path separators,
backslashes, dot bytes, controls, non-ASCII bytes, fragments, and malformed
percent escapes are rejected before credentials are attached.

## Secret Boundary

`BearerToken` copies the complete authorization value into adapter-owned
storage, marks reqwest's header value sensitive, redacts diagnostics, and
volatile-clears its owned bytes on drop through `cloud-sdk-sanitization`.
Request-body copies and caller response buffers are also cleared on their
owned failure or drop paths.

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
blocking adapter requires a reqwest/rustls-supported std target, platform trust
roots, networking, threads, and the native build requirements of aws-lc-sys.
Linux, Windows, macOS, BSD, Android, and iOS remain intended targets, but a
platform is not claimed as release-verified until its target check or CI job is
recorded. Aesynx compatibility remains a future no_std integration concern and
does not enable this std adapter.

## Verification

`scripts/check_reqwest_boundary.sh` verifies the exact top-level versions,
default and std graph isolation, required and forbidden reqwest features,
absence of native TLS and decompression dependencies, direct-zeroize
exclusion, hardened builder policy, focused tests, and package verification.
The v0.16 release gate additionally runs the full workspace checks, MSRV
matrix, cargo-deny, cargo-audit, upstream API drift checks, and pentest evidence
validation.
