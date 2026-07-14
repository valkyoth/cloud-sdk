# Release Runbook

This is the operational summary for an existing milestone. The normative
requirements and pentest evidence model remain in the
[release plan](RELEASE_PLAN.md).

## Implementation Stop

1. Finish code, tests, docs, crate versions, release metadata, lockfiles, and
   both SBOMs.
2. Confirm `release-crates.toml` publishes `cloud-sdk` at the tag version and
   only publishes another crate when its declared change requires it.
3. Run `scripts/checks.sh`, the version-specific release gate prerequisites,
   `cargo deny check`, and `cargo audit`.
4. Commit the complete implementation-stop state and push it for GitHub CI and
   CodeQL default setup.
5. Do not change release-sensitive files while that commit is under pentest.

## Pentest And Retest

1. Record temporary findings in root `PENTEST.md`; never commit that file.
2. Fix findings, add regression tests, update documentation, remove
   `PENTEST.md`, regenerate SBOMs, and repeat all local checks.
3. Commit and push the new implementation state whenever a fix changes it.
4. After a green retest, add `security/pentest/vX.Y.Z.md` with the exact full
   implementation `Reviewed-Commit`, `Status: PASS`, tester, scope, and date.
5. Commit only that permanent report as the direct child of the reviewed
   implementation commit.

## Tag Gate

After GitHub CI and CodeQL are green on the report commit:

```sh
scripts/release_0_N_gate.sh
```

The gate must begin and end at one clean unchanged `HEAD`. It validates the
report-parent relationship, runs local and live drift checks, checks SBOM
freshness, and requires the pinned dependency security tools.

Create and push a signed annotated tag only after the maintainer explicitly
approves tagging. The publisher verifies that the tag points at `HEAD`, reads
the independent publish plan from `release-crates.toml`, and must not revive
retired provider-specific helper crates.

## Failure Handling

- Any code, documentation, manifest, lockfile, workflow, release script, or
  SBOM change invalidates the reviewed implementation commit.
- A failed CI runner may be retriggered with an empty commit when GitHub does
  not allow reruns; the permanent report remains the only release-evidence
  change after the reviewed parent.
- A retest or CodeQL finding requires a new implementation commit, fresh
  review, and updated `Reviewed-Commit`.
- Never bypass dirty-tree, pentest, signed-tag, dependency, drift, or SBOM
  checks to complete a release.
