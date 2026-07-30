# v0.42.0 Dependency Review

Date: 2026-07-30

Scope: Basic authorization encoding and canonical signing inputs.

## Result

One dependency is admitted:

| Crate | Version | Use | Default graph |
| --- | --- | --- | --- |
| `base64-ng` | `1.3.9` exact | RFC 4648 padded Basic authorization encoding | absent |

Live crates.io metadata on 2026-07-30 confirmed `1.3.9` is current, uses
`MIT OR Apache-2.0`, declares Rust 1.90, and has no normal dependencies.
The reviewed crate archive SHA-256 is
`1b8a8323341659decbfed54fbe89805337845c7f0ab847fd59ffd3b7239eda9a`.

Default features are disabled. The workspace enables neither `alloc`, `std`,
SIMD, streaming, Tokio, fuzzing, nor Kani features. Only the existing explicit
blocking or async reqwest transport features activate the dependency.
`cloud-sdk-reqwest` uses `checked_encoded_len` and caller-buffer
`STANDARD.encode_slice`; it does not use decoding or allocation convenience.

The selected scalar no-feature path adds no transitive dependency and no
unsafe feature. Adapter-owned input, intermediate, encoded, and header
allocations retain separate cleanup ownership through
`cloud-sdk-sanitization`.

Core signing input construction adds no dependency and remains allocation-free
and `no_std`.

## Required Verification

- default and std-only dependency graphs exclude `base64-ng`;
- every reqwest transport feature includes exact `base64-ng 1.3.9`;
- RFC 7617/RFC 4648 vectors, exact bounds, source cleanup, intermediate
  cleanup, header sensitivity, and redacted diagnostics pass;
- Cargo Deny, RustSec, package, platform, MSRV, and SBOM checks pass;
- `scripts/check_basic_and_signing.sh`;
- `scripts/release_0_42_gate.sh` after pentest evidence is committed.
