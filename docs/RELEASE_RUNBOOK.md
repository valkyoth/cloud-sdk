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
4. Commit the complete implementation-stop state for pentest.
5. Do not change release-sensitive files while that commit is under pentest.

## Pentest And Retest

1. Record temporary findings in root `PENTEST.md`; never commit that file.
2. Fix findings, add regression tests, update documentation, remove
   `PENTEST.md`, regenerate SBOMs, and repeat all local checks.
3. Commit the new implementation state whenever a fix changes it, then repeat
   pentest.
4. After a green retest, add `security/pentest/vX.Y.Z.md` with the exact full
   implementation `Reviewed-Commit`, `Status: PASS`, tester, scope, and date.
   When the first pentest has no findings, document that result directly; a
   redundant retest is not required.
5. Commit the permanent report together with any final release metadata. The
   reviewed implementation commit must remain an ancestor of this commit.

## Tag Gate

After GitHub CI and CodeQL are green on the final release commit:

```sh
scripts/release_0_N_gate.sh
```

The gate must begin and end at one clean unchanged `HEAD`. It validates that
the reviewed commit is an ancestor of the release commit, runs local and live
drift checks, checks SBOM freshness, and requires the pinned dependency
security tools.

Create and push a signed annotated tag only after the maintainer explicitly
approves tagging. The publisher verifies that the tag points at `HEAD`, reads
the independent publish plan from `release-crates.toml`, and must not revive
retired provider-specific helper crates. It always refreshes `cargo audit`
after tag verification and before confirmation so advisories disclosed after
the release gate still block publication. It does not rerun the complete gate
by default because the signed tag already binds the unchanged commit that
passed that gate and GitHub checks. Use `scripts/release_crates.py --rerun-gate`
only when an intentional second, network-sensitive gate run is required.

`git verify-tag` uses the release operator's configured Git signature trust.
The publisher proves tag integrity, not maintainer identity through a
repository-pinned fingerprint; signer authorization and key rotation remain
release-host responsibilities.

## Failure Handling

- If CI or CodeQL finds an issue, fix it, update the report to describe the
  change and latest reviewed state, commit, and wait for GitHub again.
- A failed CI runner may be retriggered with an empty commit when GitHub does
  not allow reruns; document the operational-only commit in the report.
- A pentest finding requires a new implementation commit, retest, and updated
  `Reviewed-Commit`.
- Never bypass dirty-tree, pentest, signed-tag, dependency, drift, or SBOM
  checks to complete a release.
