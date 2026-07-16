# Reqwest Transport Dependency Admission

Status: admitted only through `cloud-sdk-reqwest/blocking-rustls`,
`cloud-sdk-reqwest/blocking-rustls-webpki-roots`,
`cloud-sdk-reqwest/blocking-rustls-fips`, and `cloud-sdk-reqwest/async-rustls`,
with reqwest default features disabled.

## Decision

| Crate | Version | Role | Default features |
| --- | --- | --- | --- |
| `reqwest` | `0.13.4` | blocking/async HTTP client and URL/header types | disabled |
| `bytes` | `1.12.1` | sanitized owned async request-body handoff | disabled |
| `hyper` | `1.10.1` | transitive HTTP implementation | transitive |
| `tokio` | `1.52.3` | reqwest runtime; direct dev-only async test executor | transitive/disabled |
| `url` | `2.5.8` | authority-preserving endpoint parsing | transitive |
| `rustls` | `0.23.42` | TLS implementation | transitive |
| `rustls-platform-verifier` | `0.7.0` | platform trust-store verification | transitive |
| `webpki-roots` | `1.0.8` | deterministic Mozilla trust-root snapshot | disabled |
| `aws-lc-rs` | `1.17.1` | rustls cryptographic provider | transitive |
| `cloud-sdk-sanitization` | `0.14.0` | adapter-owned secret-buffer cleanup | disabled |
| `sanitization` | `1.2.4` | reviewed volatile cleanup primitive | disabled |

The exact repository graph is pinned by `Cargo.lock`, checked by `cargo deny`,
and recorded in the generated SBOM. Applications own their downstream
resolution and must retain a reviewed lockfile or vendored source set; a
library lockfile is not published as a consumer constraint. All admitted
licenses satisfy `deny.toml`. The rustls trust-root data requires
`CDLA-Permissive-2.0`, which is explicitly admitted. The ordinary transport
graph has no duplicate-version exception. The FIPS native build graph narrowly skips build-only
`shlex 1.3.0` because bindgen and cc require different major lines; the
boundary still rejects legacy `windows-sys` `0.52.0` if it becomes reachable
again.

Aws-lc-sys introduces the workspace's first native dependency build script. It
invokes a C/CMake toolchain to compile vendored C and assembly cryptographic
code. Cargo authenticates the published crate archive against the checksum
pinned in `Cargo.lock`; this does not remove the trust placed in the crate's
build script, bundled source, compiler, assembler, CMake, linker, or build
host. Release CI and reproducible/offline builders must use pinned, audited
build images and toolchains. Offline preparation must preserve Cargo's
authenticated package checksum rather than copying unauthenticated source
trees. The v0.24 review and exact archive checksums are recorded in
[`DEPENDENCY_REVIEW_0.24.0.md`](DEPENDENCY_REVIEW_0.24.0.md).

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

`blocking-rustls-fips` enables the blocking and sanitization boundaries with
`reqwest/rustls-no-provider`, plus direct rustls FIPS and platform-verifier
configuration. Its additional native graph, runtime checks, validation-status
limits, and build requirements are reviewed separately in
[`dependency-admission-reqwest-fips.md`](dependency-admission-reqwest-fips.md).

`blocking-rustls-webpki-roots` enables the blocking and sanitization
boundaries with an explicit AWS-LC provider, direct rustls configuration, and
`webpki-roots 1.0.8`. Its client receives only the compiled Mozilla snapshot.
Reqwest still compiles its platform-verifier graph, but that verifier is not
installed into this preconfigured client. Root changes require a dependency
update; host enterprise roots, private PKI, revocation checking, and pinning
are not provided by this mode.

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
dependency is pinned exactly to `0.20.2` and enables the standard, FIPS, and
async transport features, proving the explicit FIPS configuration wins under
additive feature unification. The deterministic-root boundary separately
compiles both its standard combination and its combination with FIPS.

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
- strict all-or-none decimal parsing of exactly one of each of the three
  provider-neutral rate-limit response metadata fields;
- async response accumulation limited to caller capacity, followed by one
  complete-success copy into the caller buffer.

The blocking and async clients are cloneable shared handles. Their token store
uses a short-lived standard-library read/write lock over reference-counted
token snapshots. No lock is held while reqwest performs network I/O or while an
async request is suspended. The SDK provides no queue, semaphore, task set,
retry fan-out, sleep, or background rotation worker; applications select and
enforce their own concurrency bounds. A poisoned lock is recovered while its
guard still protects one structurally complete token `Arc`, so an unwind cannot
permanently disable all cloned clients.

The adapter validates the final scheme, host, and port against the configured
endpoint. Request targets are origin-form, and encoded path separators,
backslashes, dot bytes, controls, non-ASCII bytes, fragments, and malformed
percent escapes are rejected before credentials are attached.

Every built client also implements `BoundTransport` and reports a normalized,
credential-free scheme, host, effective port, and base path. The endpoint is
owned immutably by every clone and cannot be replaced through rotation. This
identity enables provider facades to fail closed on official-host, subdomain,
port, scheme, or base-path mismatches. `cloud-sdk-hetzner` supplies this exact
check for both official v1 endpoint families.

Standard transport roots intentionally follow host policy. A compromised or
attacker-extended host trust store, including an enterprise interception root,
can therefore validate a hostile endpoint. The separately reviewed
`blocking-rustls-webpki-roots` feature avoids host-added roots with a compiled
Mozilla snapshot, but it does not provide revocation checks or certificate or
public-key pinning.

The `blocking-rustls-fips` transport explicitly selects and verifies the rustls
FIPS provider and complete client configuration. It does not trust unrelated
dependency features or process-global provider state, and it is not a broader
application or deployment compliance guarantee.

## Secret Boundary

`BearerToken` copies the complete authorization value into adapter-owned
storage, marks reqwest's header value sensitive, redacts diagnostics, and
volatile-clears its owned bytes on drop through `cloud-sdk-sanitization`.
Mutable-byte and guarded-buffer constructors clear their complete caller-owned
source on success or rejection. Rotation validates and allocates a replacement
before changing shared state, so rejected input leaves the active token intact.
In-flight requests retain reference-counted old-token snapshots; retired
adapter storage drops and clears only after the last snapshot finishes.
Request-body copies and caller response buffers are also cleared on their
owned failure or drop paths. Async response data is accumulated in a
pre-reserved adapter-owned sanitized buffer and copied into the already-cleared
caller buffer only after complete success. Dropping the future during connect,
send, or response reading drops and clears that adapter-owned accumulation.

The adapter cannot clear a caller's original immutable token string,
reqwest/header/TLS copies, operating-system socket buffers, kernel memory,
swap, crash dumps, or remote systems. Callers must generate tokens with an
appropriate CSPRNG, rotate and revoke them operationally, minimize scope, and
prefer the source-clearing constructors when mutable storage is available.

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

`scripts/check_reqwest_boundary.sh`,
`scripts/check_reqwest_webpki_roots_boundary.sh`, and
`scripts/check_reqwest_fips_boundary.sh` verify the exact top-level versions,
default and std graph isolation, separate blocking/async required and forbidden
reqwest features,
absence of native TLS and decompression dependencies, direct-zeroize
exclusion, hardened builder policy, adversarial HTTP/2/Hickory feature
unification, focused tests, fixture lockfile policy and audit coverage,
package verification, the FIPS graph, and runtime FIPS status.
Every entry point that can compile AWS-LC also sources the shared bundled-build
policy, which rejects target-qualified system-library overrides and forces the
ordinary and FIPS generic system controls off.
The release gate additionally runs the full workspace checks, MSRV
matrix, cargo-deny, cargo-audit, upstream API drift checks, and pentest evidence
validation.
