# v0.32.0 Dependency Review

Date: 2026-07-26

Scope: direct and locked dependency freshness during the v0.32 provider
identity release. This review changes no default `cloud-sdk` dependency graph.

## Reviewed Direct Updates

| Package | Previous | Reviewed | Rust | Scope |
| --- | --- | --- | --- | --- |
| `sanitization` | `1.2.5` | `2.0.3` | 1.90 | optional first-party cleanup boundary |
| `tokio` | `1.53.0` | `1.53.1` | 1.71 | optional reqwest runtime and tests |
| `syn` | `3.0.2` | `3.0.3` | 1.71 | excluded source checker |

All other direct package pins were checked against crates.io on the review
date. The stable rustls line remains `0.23.42`; the crates.io search result for
`0.24.0-dev.1` is a development release and is not admitted as a stable
upgrade. Cargo release tools remain at their currently pinned versions.

## sanitization 2.0.3

The first-party major release retains the exact surface used by this workspace:

- Rust 1.90 and `no_std`;
- `wipe::bytes` volatile clearing, retained behind this workspace's existing
  `sanitize_bytes` wrapper;
- `SecretString` behind `alloc`;
- owned `String` ingestion without another plaintext allocation;
- closure-scoped immutable and mutable UTF-8 access;
- old-allocation clearing before growth;
- full allocation-capacity cleanup on clear and drop.

The workspace disables upstream defaults, including `asm-compare`. It enables
only `alloc` through `cloud-sdk-sanitization/alloc`. The admitted default and
alloc graphs contain no transitive package. Interoperability, derive, Serde,
memory locking, guard pages, cache flushing, canaries, hardware secrets,
register scrubbing, split secrets, and multi-pass clearing remain disabled.

The public `cloud-sdk-sanitization` wrapper surface is unchanged. Existing
guard, parser, async cancellation, credential rotation, request-body, and
failure-path tests exercise the upgraded primitive.

## Isolated And Transitive Updates

Tokio remains default-feature-free and is enabled only by the optional async
reqwest boundary or test fixture. The direct isolated `syn` checker still
enables only `full`, `parsing`, and `visit`; it remains unpublished with its
own lockfile.

Refreshing independent lockfiles also selected compatible transitive releases
in the excluded fuzz and reqwest feature-unification graphs. Those complete
graphs remain independently covered by Cargo Deny, RustSec, and SPDX evidence.

## Cargo Archive Checksums

| Archive | SHA-256 from lockfile |
| --- | --- |
| `sanitization 2.0.3` | `75e43f2762b31232062e8ba7bfbdfcbd33c80c43bf7a306a7e195c3c4f734e0f` |
| `tokio 1.53.1` | `202caea871b69668250d242070849eb495be178ed697a3e98aebce5bc81a0bed` |
| `syn 3.0.3` | `53e9bae58849f64dfa4f5d5ae372c8341f7305f82a3868709269343628b659a3` |

Cargo authenticates these archives against their lockfile checksums. This does
not remove trust in proc macros, native build inputs, the compiler, or the
release host.

## Required Verification

- `scripts/check_sanitization_boundary.sh`
- `scripts/check_reqwest_boundary.sh`
- isolated prepared-coverage checker tests and clippy
- all four Cargo Deny and RustSec graph checks
- all four complete SPDX freshness checks
- Rust 1.90.0 through 1.97.1 compatibility
- `scripts/checks.sh`
