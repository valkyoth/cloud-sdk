# crates.io Candidate Qualification

Status: full-scope review and remediation retest accepted at `96ae9a2c`.
Release: workspace `1.1.0`; final local/GitHub qualification and publication approval remain required.
Previous accepted checkpoint: `cbcaf17f54b50918b9b6f6fb29f76cbd9656b537`.

The complete candidate gate passed on 2026-09-26 at
`dd753e9372ad5af99b7e6274402303f446c93132`, including identical archives for
all six packages from two clean clones. Subsequent documentation-only evidence
updates do not constitute pentest acceptance. Warning-denied workspace rustdoc,
live IANA IPv6 and Robot wire-source checks also passed.

The subsequent full review found two availability issues; both were fixed and
independently retested at `96ae9a2c`, with no new confirmed finding. The
[permanent release report](../security/pentest/v1.1.0.md) distinguishes the
full assessment from its focused remediation retest. Release preparation now
reruns `scripts/release_1_1_gate.sh` on the committed publication metadata.
Final tooling qualification also covers all workspace patches in the legacy
archive/SBOM reproducer. Live governance checks require existing crate ownership;
only an explicitly initial package with Cargo's exact namespace-absent 404 is
reported as awaiting first publication. Any other retrieval or ownership failure
remains fatal. Recheck the new package's owner immediately after its first upload.

## Frozen Scope

The [51-operation scope matrix](CRATESIO_API_SCOPE.tsv) is authoritative.
All selected rows have named executable clients; none is model-only. Seven
operations overlap the stable Cargo Registry Web API. Other public crates.io
operations remain provider-specific/experimental, not promises of upstream
stability. The two upstream-deprecated public rows remain explicitly classified.
New upstream operations require a new scope decision rather than automatic admission.

The [provider README](../crates/cloud-sdk-cratesio/README.md) contains examples
and feature selection. Execution includes blocking, local async and Send async;
bounded buffered and streaming paths; credential-bound mutations; Cargo publication
framing; owner/yank workflows; and verified transactional archive downloads.
Empty defaults retain the transport-free boundary. The SDK does not implement
Cargo packaging, a registry server, browser sessions, bulk index synchronization,
or automatic mutation retries. Source-lock exclusions are not implementation gaps
silently marked supported.

## Documentation And Security

- [Authentication](CRATESIO_CREDENTIAL_POLICY.md) and the provider README describe
  API-token and trusted-publishing boundaries; only official origins receive credentials.
- [Qualification evidence](CRATESIO_QUALIFICATION.md) distinguishes production
  probes from fixture coverage, including the authenticated-following 403 and
  operator-confirmed cleanup. No new live mutation is part of this candidate gate.
- [Download policy](CRATESIO_DOWNLOAD_POLICY.md) and
  [live-read policy](CRATESIO_LIVE_READ.md) retain their explicit trusted-adapter,
  storage, checksum, resource-limit and operator boundaries.
- [Source lock](CRATESIO_SOURCE_LOCK.md), [commit plan](cratesio-commit-plan.md),
  [platform policy](PLATFORM_SUPPORT.md), and
  [release notes](../release-notes/RELEASE_NOTES_1.1.0.md) remain candidate documents.

No new cryptography, unsafe code, implicit retry, or mutation
authority is introduced by Commit 22. Network members are read-only. Responses
do not echo their parent Network; the authenticated exchange provides that
association. Type/status strings retain forward-compatible values, IPv4 fields
are validated, and normal response/body/allocation bounds still apply. Schema
updates cannot silently relax endpoint or credential policy.

The first full-service review found blocking-runtime shutdown and source-fetch
deadline availability issues. Remediation adds non-waiting runtime cleanup,
bounded resolver jobs, and a killable source-fetch worker. The existing
transitive `tower-service` package is now an optional direct transport dependency;
see its admission record. Earlier qualification does not qualify these changes.

## Compatibility Review

The existing public `NetworkEndpoint` and `SourceQueryParameter` enums retain
their exhaustive variants. New functionality is additive through
`NetworkMembersEndpoint`, `SourceQueryArgument::subnet`, the new generated
operation marker and three client methods. `NetworkMember` is added only to
already non-exhaustive response enums. Existing query constructors and validation
remain available. The private query argument key representation changes without
exposing caller-controlled parameter names. Primary IP response cross-field
validation was already present; only the source enum evidence changes.

The automated SemVer comparison is supplemented by external-consumer tests:
the existing incremental JSON types moved into the neutral crate in Commit 6
and remain reexported at their published Hetzner paths. `cargo-semver-checks`
0.49.0 reports those external reexports as missing enum/struct/trait definitions;
the compile/run witness imports every reported name, implements the visitor and
executes the decoder. These specific reports are reviewed tool limitations, not
blanket permission to ignore future failures. A discriminant regression test
also locks all twelve published `CloudResourceKind` values; additions append.

The candidate review also corrected four exhaustive `RawHttpError` additions
introduced earlier in this train. URI staging failures use the existing
`RequestBuildFailed`; invalid streaming state, upload failure and premature
response use `RequestFailed`. Existing delivery-phase wrappers, abort/cleanup,
and no-retry decisions are unchanged. Exhaustive-match/discriminant regression
tests preserve the 1.0 error contract, and reqwest's full-feature SemVer scan
against `v1.0.0` passes. The neutral, sanitization and testkit scans also pass.

The Hetzner [source review](SPEC_LOCK.md#reviewed-live-drift-2026-09-26) resolves
the known September drift; the [changelog review](HETZNER_CHANGELOG_LOCK.md)
retains the upcoming nullable TTL/PTR, Data Center and legacy deprecation-field
notices. No retired operation is reintroduced.

## Reproducible Gate

From a clean committed checkout:

```sh
scripts/check_cratesio_candidate.sh
```

This composes the full repository checks (including all preceding incremental
contracts), 39-target fuzz smoke, every admitted Rust version and platform,
six packaged feature graphs, advisory/deny/SBOM checks, live crates.io and
Hetzner drift, and tool/dependency freshness. It then packages all six crates
from two independent clean clones of the same HEAD and compares archive bytes
and members. Archive creation does not publish and does not replace package
compilation, which is separately mandatory. The clone gate rejects dirty roots,
dirty clones, changed HEAD, missing packages and mismatched artifacts.

Pentest and GitHub acceptance are intentionally not inferred from local checks.
After qualification, stop for full-service pentest covering `v1.0.0..HEAD`;
use `cbcaf17f..HEAD` for the incremental Commit 22 review. Remediate/retest,
run the release gate, wait for GitHub CI/CodeQL, then obtain an explicit release
decision before changing candidate status, tagging or publishing.
