# Release Governance

Status: reviewed for the v0.99.0 provenance and governance milestone.

This document defines how a maintainer verifies, recovers, reproduces, and
releases the exact reviewed source. It does not claim organizationally
independent governance, accredited build provenance, or protection from a
compromised release host.

## Authority Boundaries

`release-governance.toml` is the machine-readable package and workflow policy.
`scripts/check_release_governance.py` enforces its source-controlled portion.
Only these packages may ever reach the publisher:

1. `cloud-sdk-sanitization`
2. `cloud-sdk`
3. `cloud-sdk-reqwest`
4. `cloud-sdk-testkit`
5. `cloud-sdk-hetzner`

The OVHcloud probe, fuzz package, feature-unification harness, and coverage
tool must set `publish = false`. A new manifest fails governance checks until
it is explicitly classified. `scripts/release_crates.py` has the same closed
allowlist, rejects retired and nested provider packages, derives the release
selection from `release-crates.toml`, and invokes only `cargo publish --locked
-p NAME`. It contains no Git push or GitHub release operation.

An internal milestone must select no crate. At a public checkpoint, cumulative
Git changes from the preceding public tag and dependency closure determine
which allowed packages need version changes. The facade always follows the
workspace release tag and publishes last among packages that depend on it.

## GitHub Review

The following controls were read through the GitHub API on 2026-08-19. Run
`scripts/check_release_governance.py --live` before v0.99 and every public
checkpoint to detect drift.

| Control | Reviewed state | Boundary |
| --- | --- | --- |
| Default branch | `main` | Repository setting, not source-controlled. |
| Branch ruleset | Active for the default branch | Prevents creation, deletion, and non-fast-forward updates; requires linear history, signed commits, pull-request review, CODEOWNERS review, stale-review dismissal, last-push approval, and CodeQL. |
| Ruleset bypass | Maintainer and organization-administrator bypass exists | Direct maintainer releases are procedurally reviewed, not independently prevented by GitHub. |
| Tag ruleset | None | Signed annotated tags, local gates, exact target verification, and explicit maintainer approval are procedural controls. |
| Actions token | Repository default is read-only and cannot approve reviews | Every committed workflow also declares only `contents: read`. |
| Allowed Actions | GitHub permits all Actions and does not require SHA pins | The source checker rejects every non-SHA-pinned `uses:` entry and every unclassified workflow. |
| CodeQL | Default setup, default suite, weekly, Actions/Python/Rust | No advanced CodeQL workflow is maintained in this repository. |
| Secret scanning | Disabled in the reviewed repository settings | Repository checks prohibit release credentials in workflows, but GitHub secret scanning and push protection are not claimed. |

The source checker also rejects job-level permission overrides, write or OIDC
permissions, release-triggered workflows, `cargo publish`, and `gh release` in
GitHub Actions. CI therefore cannot publish a crate or mint a trusted-publisher
token from this repository configuration.

## Trusted Publishing Decision

Trusted publishing is not enabled for the pre-1.0 release train. Publication
is an explicit local operator action after a signed tag, green GitHub checks,
and a clean release gate. `cargo` obtains its crates.io credential from the
release host; the repository, release plan, logs, and GitHub workflows must not
contain that credential. The operator rotates or revokes it through crates.io
account settings.

This decision avoids granting a GitHub workflow publication authority while
the workflow and governance model are still pre-1.0. A future migration to
trusted publishing requires its own threat-model change, environment and
subject binding, minimal OIDC permission, publisher tests, documentation, and
pentest. It must remove the superseded long-lived token rather than retain two
unreviewed publication paths.

## Signer Lifecycle

Release tags and release commits use the operator's configured SSH signing
key. The repository intentionally does not contain a private key, a pentest
report-signing key, or a repository-pinned list that could falsely imply
organizational authorization. `git verify-tag` proves integrity against the
release host's configured trust; operator authorization and key custody remain
host and GitHub account responsibilities.

Normal rotation:

1. Generate a new Ed25519 signing key with an operating-system CSPRNG on the
   controlled release host and protect it with the host's credential policy.
2. Add the public signing key to GitHub and the operator's trusted-signers
   configuration before using it.
3. Create a disposable signed object, verify it from a separate clean clone,
   and remove only that disposable object after verification.
4. Switch `user.signingkey`, make the next release commit and tag with the new
   key, and retain old public keys while historical signatures need to verify.
5. Remove the old private key from agents and release hosts after the overlap
   window. Record the rotation without exposing key material.

Suspected compromise:

1. Stop commits, tags, releases, and crate publication immediately.
2. Remove the compromised public key from GitHub, revoke host access, rotate
   GitHub and crates.io credentials, and preserve logs and objects for review.
3. Audit every commit, tag, GitHub release, crate version, owner change, and
   workflow run since the last known-good event from a separate clean host.
4. Never move or overwrite a published tag. Publish a security advisory and a
   new corrective version; yank an affected crate version when appropriate.
5. Resume only after a new key passes the normal rotation drill and the exact
   repaired commit repeats pentest, release gates, CI, and CodeQL.

## Ownership And Repository Recovery

The 2026-08-19 crates.io review found the required `eldryoth` owner on all five
publishable crates. It is currently the only listed owner, so loss of that
account has no independent owner fallback. Recovery depends on crates.io
account recovery or crates.io support and proof of control over the public
repository. This limitation must not be described as redundant ownership.

Before changing owners, verify the proposed user or GitHub team out of band,
add it to one crate, confirm visibility and least privilege, and then repeat
for each package. A removal follows the same two-person verification where a
second owner exists. Owner commands must never be embedded in the publisher.

Repository recovery drill:

1. Clone the canonical GitHub repository into a new path and fetch all tags.
2. Compare the expected remote URL and default branch, then verify the latest
   signed tag and its exact commit using separately trusted public-key data.
3. Run `scripts/check_release_governance.py`, `scripts/validate-release-train.py`,
   `scripts/check_sbom_freshness.sh`, and the applicable release gate.
4. Run `scripts/check_release_provenance.py`. It creates two further clean
   clones and must reproduce all package archives and canonical complete SBOMs.
5. Restore GitHub rulesets, CodeQL default setup, Actions defaults, secrets,
   crates.io ownership, and release-host credentials through their providers;
   none are recoverable from repository contents alone.

The procedure proves that committed source is sufficient to reconstruct
artifacts. It does not make external account configuration part of Git.

## Reproducible Evidence

`scripts/check_release_provenance.py` refuses a dirty source tree, clones the
exact commit twice without local hard links, and from each clone:

- creates all five allowed `.crate` archives with `--locked --no-verify`;
- resolves unpublished in-train first-party versions through an explicit,
  package-specific local patch inventory while retaining registry dependency
  declarations in the resulting archives;
- generates all four complete SPDX dependency graphs;
- compares package archives byte for byte through SHA-256;
- compares SBOMs after removing only cargo-sbom's creation timestamp and random
  document namespace and normalizing the validated checkout-basename document
  name to its logical project name; and
- compares each generated canonical SBOM with committed evidence.

The report prints the source commit and tree, `Cargo.lock` SHA-256, package and
canonical-SBOM SHA-256 values, and exact Git, Rust, Cargo, and cargo-sbom
versions. Package compilation and tests remain separate mandatory gates;
`--no-verify` here isolates archive construction from compilation so the two
pieces of evidence cannot conceal each other.

## Rollback And Incident Response

Git tags and crates.io artifacts are immutable evidence. Do not force-update a
release tag, overwrite a GitHub release, or attempt to reuse a crates.io
version. For an unpublished candidate, stop and create a new reviewed commit.
For a published defect:

1. stop further publication and classify affected versions;
2. publish an advisory when security-relevant;
3. yank only versions whose continued selection is unsafe or materially
   broken, understanding that yanking does not delete downloaded artifacts;
4. create a new patch or planned release from the last known-good ancestry;
5. repeat dependency, provenance, pentest, CI, CodeQL, tag, and publication
   controls; and
6. record what was yanked, superseded, or left available and why.

Infrastructure changes made through the SDK are outside release rollback. They
require provider-specific reconciliation and cleanup under the operation's
delivery and cost policy.

## Review Independence

Repository pentests performed by the maintainer are security review, but they
are not organizationally independent assessments. GitHub CodeQL, dependency
advisories, fuzzing, and release automation are additional evidence, not a
substitute for an independent assessor or deployment accreditation. Pentest
reports remain ordinary committed Markdown evidence and are not separately
signed. The signed release tag binds the complete commit containing them.

Claims such as FIPS validation, military-grade security, regulated-environment
approval, reproducible operating environments, or independent certification
are outside the current project evidence.
