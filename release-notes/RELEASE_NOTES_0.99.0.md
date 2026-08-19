# cloud-sdk 0.99.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-19

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.100.0

## Overview

v0.99 makes the release trust boundary explicit and executable. It qualifies
package eligibility, workflow authority, signer and owner recovery, immutable
rollback, clean-checkout reconstruction, and package/SBOM reproducibility. It
is an internal cumulative tag and publishes no crate.

## Governance And Recovery

- `release-governance.toml` names exactly five publishable packages and four
  excluded packages. Every new Cargo manifest must be classified.
- Source checks require one known workflow inventory, SHA-pinned Actions,
  top-level `contents: read`, no elevated job permissions or execution
  modifiers, and exact reviewed expression, action, action-input, step, command,
  environment, GitHub-hosted runner, and runner-matrix forms.
- Pentest remediation inventories both workflow extensions and uses a bounded
  event pass to reject anchors and aliases before YAML DOM construction. Flow
  mappings, merge keys, job overrides, action references, triggers, event and
  depth exhaustion, custom shells, alternate action inputs, credential syntax,
  indirect command construction, self-hosted labels, runner arrays, hostile
  matrix values, and matrix include overrides fail closed.
- Live review compares the exact source-locked default branch, branch ruleset
  and bypass actors, tag ruleset state, Actions policy, CodeQL setup, secret
  scanning state, zero direct self-hosted runners, zero inherited runner groups
  visible to the repository, and crates.io ownership before reporting success.
- The governance review records current limitations: maintainer and
  organization-administrator branch bypasses, no tag ruleset, repository-level
  SHA pin enforcement disabled, GitHub secret scanning disabled, and one
  crates.io owner per package.
- Signer rotation, compromise response, repository/account recovery, crates.io
  recovery, credential rotation, immutable tags, yanking, and corrective
  releases have explicit procedures.
- Pentest evidence is bound by the signed release commit but is not separately
  signed and is not described as organizationally independent.

## Provenance And Publication

- Two independent clean clones of one exact commit build every allowed crate
  archive using the lockfile and explicit package-specific local patches for
  unpublished in-train dependencies, then regenerate all four complete SPDX
  graphs.
- Package archives must have identical SHA-256 values. Canonical SBOMs remove
  creation time and random namespace and normalize only the validated
  checkout-basename document name before comparison; they must also equal
  committed evidence.
- Evidence prints the source commit/tree, `Cargo.lock` digest, exact Git/Rust/
  Cargo/cargo-sbom versions, and every package and canonical-SBOM digest.
- Package policy, committed SBOMs, the source tree, and `Cargo.lock` are read
  from the captured Git object; changed source state prevents final success.
- The publisher now rejects direct calls for any package outside its closed
  allowlist. Its exclusion regression runs as part of the normal test suite.
- `release_crates.py --check --version 0.99.0` must report `stage=internal`, and
  every crate entry keeps `publish = false`.

## Versions

| Crate | Published | v0.99 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.95.0` | `0.99.0` | deferred |
| `cloud-sdk-hetzner` | `0.46.0` | `0.46.0` | accumulated code; deferred |
| `cloud-sdk-reqwest` | `0.36.0` | `0.36.0` | accumulated code; deferred |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.31.0` | `0.31.0` | accumulated dependency change; deferred |

## Pentest Remediation

- Replaced line-oriented workflow inspection with bounded YAML event and DOM
  parsing, then restricted workflow expressions, actions, inputs, steps,
  commands, environments, execution modifiers, and runner selectors to exact
  reviewed forms.
- Added a closed GitHub-hosted runner matrix and independent live zero baselines
  for direct repository runners and organization or enterprise runner groups
  visible to this repository.
- Bound provenance inputs to the captured Git object, rejected concurrent
  source-state changes, and made clean-clone package and canonical SBOM
  reproduction deterministic for unpublished in-train dependencies.
- Expanded regression coverage for workflow authority bypasses, parser resource
  bounds, release selection, live governance drift, and artifact provenance.

## Stop Gate

The incremental pentest and remediation retest are green. Run
`scripts/release_0_99_gate.sh` against the exact final evidence commit and
require green GitHub CI and CodeQL before tagging. Do not publish crates; the
cumulative public checkpoint is v0.100.0.

## Result

v0.99.0 is ready for its internal signed tag after the clean local release
gate and GitHub CI and CodeQL pass. No crate is selected for publication.
