# Release Runbook

This is the operational summary for an existing milestone. The normative
requirements and pentest evidence model remain in the
[release plan](RELEASE_PLAN.md).

## Implementation Stop

1. Finish code, tests, docs, crate versions, release metadata, lockfiles, and
   all SBOMs.
2. Set `release-crates.toml` to `stage = "internal"` for an ordinary
   intermediate tag or `stage = "public"` for a scheduled or exceptional
   checkpoint. Keep the normal `vX.Y.Z` tag name in either case.
3. Keep `baseline` at the preceding public checkpoint and list every minor and
   patch tag after it through the candidate in `cumulative_milestones`.
4. For an internal stage, select no crates for publication and retain the
   published versions of supporting crates. For a public stage, cumulatively
   classify every changed package and publish the facade last.
5. Run `scripts/checks.sh`, the version-specific release gate prerequisites,
   `cargo deny check`, and `cargo audit`.

## Intermediate Tag

1. Record `Security-Review: PASS`, `Pentest: DEFERRED TO v0.N.0`, and
   `Publication: DEFERRED TO v0.N.0` in the release notes.
2. Do not create `security/pentest/vX.Y.Z.md`; the release readiness gate
   rejects a report that would imply an independently pentested milestone.
3. Commit the complete milestone, confirm GitHub CI and CodeQL on that exact
   commit, and run the version-specific gate.
4. Create and push the ordinary signed annotated `vX.Y.Z` tag only after the
   maintainer explicitly approves it.
5. Do not run `scripts/release_crates.py`; it refuses crates.io publication for
   `stage = "internal"`.

Every intermediate tag remains inside the cumulative range assessed at the
next checkpoint. Patch tags do not reset or advance the five-minor schedule.

## Public Checkpoint

A `v0.N.0` milestone divisible by five is the scheduled public checkpoint.
Material credential, trust, transport, parsing, destructive-operation,
unsafe/native, release-control, incident, or security-fix changes may require
an earlier exceptional assessment. A targeted assessment may remain an
unpublished tag and does not narrow the next scheduled range. Exceptional
crates.io publication requires cumulative or full assessment and establishes a
new public baseline. v1.0.0 always requires a full-project assessment.

Before pentest:

1. Confirm the release plan is `stage = "public"` and that the cumulative
   milestone list includes every tag after the baseline.
2. Run the full local and version-specific gates.
3. Commit the exact implementation-stop state for pentest.
4. Do not change release-sensitive files while that commit is under pentest.

## Pentest And Retest

1. Record temporary findings in root `PENTEST.md`; never commit that file.
2. Fix findings, add regression tests, update documentation, remove
   `PENTEST.md`, regenerate SBOMs, and repeat all local checks.
3. Commit the new implementation state whenever a fix changes it, then repeat
   pentest.
4. After a green retest, add `security/pentest/vX.Y.Z.md` with `Status: PASS`,
   the applicable assessment, baseline, range end, exact full implementation
   `Reviewed-Commit`, tester, scope, and date.
   When the first pentest has no findings, document that result directly; a
   redundant retest is not required.
5. Commit the permanent report together with any final release metadata. The
   reviewed implementation commit must remain an ancestor of this commit.

## Tag Gate

After GitHub CI and CodeQL are green on the final release commit:

```sh
scripts/release_0_N_gate.sh
```

The gate must begin and end at one clean unchanged `HEAD`. For an intermediate
tag it validates the security review and exact checkpoint deferral. For a
public checkpoint it additionally validates cumulative pentest evidence and
that the reviewed commit is an ancestor of the release commit. Both stages run
local and live drift checks, check SBOM freshness, and require the pinned
dependency security tools.

Create and push a signed annotated tag only after the maintainer explicitly
approves tagging. Only a public stage may continue to crates.io. The publisher
verifies the cumulative package plan and that the tag points at `HEAD`; it must
not revive retired provider-specific helper crates. It always refreshes
`cargo audit` after tag verification and before confirmation so advisories
disclosed after the release gate still block publication. It does not rerun
the complete gate by default because the signed tag already binds the unchanged
commit that passed that gate and GitHub checks. Use
`scripts/release_crates.py --rerun-gate` only when an intentional second,
network-sensitive gate run is required.

`git verify-tag` uses the release operator's configured Git signature trust.
The publisher proves tag integrity, not maintainer identity through a
repository-pinned fingerprint; signer authorization and key rotation remain
release-host responsibilities.

Immediately before every crate publication, the publisher rechecks that
`HEAD`, the clean worktree, annotated tag target, and tag signature still
match the originally approved commit. Every publication uses `--locked`; a
checkout, tag replacement, or filesystem change during confirmation or a
crates.io wait aborts the remaining sequence. Clean-tree checks explicitly
request all untracked files and do not trust repository Git status display
configuration.

Repository checks also extract the generated `cloud-sdk-reqwest` crate and
compile its FIPS tests with `--locked`. Public certificate and CRL verifier
fixtures used by those tests must therefore be present in the publishable
archive. Package generation and extracted-crate compilation use fresh,
script-owned target directories so inherited Cargo configuration cannot cause
a stale archive to be validated.

## Failure Handling

- If CI or CodeQL finds an issue, fix it, update the security review or public
  report as applicable, commit, and wait for GitHub again.
- A failed CI runner may be retriggered with an empty commit when GitHub does
  not allow reruns; document the operational-only commit in the report.
- A pentest finding requires a new implementation commit, retest, and updated
  `Reviewed-Commit`.
- Never bypass dirty-tree, release-stage, cumulative-range, applicable pentest,
  signed-tag, dependency, drift, or SBOM checks to complete a release.
