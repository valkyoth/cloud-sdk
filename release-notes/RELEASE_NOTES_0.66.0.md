# cloud-sdk 0.66.0 Milestone Notes

Status: implementation stop reached; pentest required.

Release date: 2026-08-08

Security-Review: PASS
Pentest: PENDING
Publication: DEFERRED TO v0.70.0

## Overview

v0.66 completes source-derived Hetzner certificate and SSH-key response
models. It is an internal tag and publishes no crate. The provider package
remains at 0.40.0 while changes accumulate for v0.70.0.

## Security Models

- Added one typed `SecurityResource` family for certificate and SSH-key
  singleton, page, and create-composite responses.
- Preserved every current certificate field, including type, validity,
  domains, fingerprint, usage, managed issuance/renewal state, and protected
  provider failure detail.
- Preserved every current SSH-key field, including labels, canonical creation
  time, a provider MD5 fingerprint bound to structurally decoded OpenSSH/RFC
  4253 key material, and an SDK-computed SHA-256 identity fingerprint.
- Rejected unknown certificate states, malformed PEM chains, status/error
  contradictions, malformed fingerprints, invalid public keys, and invalid
  timestamps before returning public models.
- Preserved exact certificate-specific failure codes alongside their generic
  `ApiErrorCode` classification, with redacted diagnostics and drop cleanup.

## Security And Verification

- Extended deterministic source evidence from 569 to 595 field-contract rows
  using the exact pinned Hetzner Cloud specification.
- Kept certificate chains and SSH public keys in protected owned storage with
  closure-scoped access, redacted diagnostics, and guarded parser error paths.
- Added source-regeneration, chain-limit, key/fingerprint, state-coherence,
  cleanup, all-operation, named fuzz-seed, vertical execution, and ignored
  read-only live-smoke coverage.
- Added `scripts/check_security_response_models.sh` to the ordinary and final
  release gates.
- Admitted exact, non-default `ssh-key 0.6.7` and `md-5 0.11.0` dependencies
  only behind the provider Serde feature. MD5 is limited to checking Hetzner's
  legacy compatibility field and is not used for new security decisions.

## Versions

| Crate | Source version | Cumulative change | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.66.0` | metadata | deferred to v0.70.0 |
| `cloud-sdk-hetzner` | `0.40.0` | code | deferred |
| `cloud-sdk-reqwest` | `0.34.0` | unchanged | no |
| `cloud-sdk-sanitization` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.0` | unchanged | no |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW_0.66.0.md`](../docs/PUBLIC_API_REVIEW_0.66.0.md)
- [`docs/DEPENDENCY_REVIEW_0.66.0.md`](../docs/DEPENDENCY_REVIEW_0.66.0.md)
- [`docs/THREAT_MODEL_DELTA_0.66.0.md`](../docs/THREAT_MODEL_DELTA_0.66.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.66.0.md`](../docs/REJECTED_ABSTRACTIONS_0.66.0.md)
- [`docs/MIGRATION_0.66.0.md`](../docs/MIGRATION_0.66.0.md)
- [`docs/dependency-admission-ssh-key.md`](../docs/dependency-admission-ssh-key.md)

## Release Gate

Pentest this exact implementation-stop commit. After remediation and a green
retest, add the permanent v0.66 report and run
`scripts/release_0_66_gate.sh` on the clean evidence commit. GitHub CI and
CodeQL must be green on that unchanged commit before the signed internal tag.
Do not publish crates.
