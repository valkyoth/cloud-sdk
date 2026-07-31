# v0.43.0 Dependency Review

Date: 2026-07-31

Scope: authenticated raw-wire migration.

## Result

No direct dependency was added, removed, or version-changed by this
implementation. `cloud-sdk`, `cloud-sdk-hetzner`, and `cloud-sdk-testkit`
retain their existing default `no_std` graphs. The reqwest adapter continues
to activate networking, Tokio, Hyper, rustls, and optional trust policy only
through explicit transport features.

Bearer and Basic clients now reuse the already admitted raw Hyper engine.
This removes duplicate high-level response execution code without broadening
the dependency graph. `base64-ng` remains limited to explicit authenticated
reqwest features, and `cloud-sdk-sanitization` retains cleanup ownership for
adapter staging and credential values.

## Required Verification

- default and std-only graphs contain no transport, TLS, runtime, or OS stack;
- all explicit reqwest feature combinations compile;
- Cargo Deny and RustSec checks pass for workspace, unification, fuzz, and
  coverage-tool locks;
- platform, MSRV, package, docs.rs, and SBOM checks pass;
- `scripts/check_hetzner_wire_migration.py`;
- `scripts/test-hetzner-wire-migration.py`;
- `scripts/release_0_43_gate.sh` after pentest evidence is committed.
