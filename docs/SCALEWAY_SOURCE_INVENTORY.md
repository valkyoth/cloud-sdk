# Scaleway Source Inventory

Status: Commit 1 candidate evidence, pending pentest and GitHub acceptance, not
an SDK support claim. Target release: tentatively 1.2.0 after the full train.
The maintainer approved later follow-up of the 20 current scope ambiguities on
2026-09-26; they do not block this checkpoint but still block final qualification.

## Captured Evidence

The 2026-09-26 public-source capture contains 94 sources (9,373,995 uncompressed
bytes), documentation build `v1.8872.0`, 91 catalog entries, 90 OpenAPI schemas,
and 1,513 operation entries. Counts include duplicate surfaces across versions
and unlisted candidates; they are not counts of implemented SDK operations.

The separate Object Storage documentation link is retained for Commit 2 instead
of silently counted as a parsed control-plane schema. The pinned Go SDK revision
is `684f67323db64f059ef53f4c53a970c0b1591a1d`. Its complete recursive tree and
Billing v2 generated source are included in the evidence.

- [Source and operation inventory](../provider-drift/scaleway/inventory.json).
- Raw evidence is stored in `provider-drift/scaleway/sources/<sha256>.gz`.
- SHA-256 and byte counts apply to uncompressed original bytes, not a reformatted
  or partially parsed schema.
- The index, flat literal catalog, nested navigation routes and independent
  schema-input registry must agree. JavaScript is inspected, never executed.
- Documented versions are retained unless explicit supersession evidence exists.
  Deprecation flags are recorded separately and do not silently remove operations.
- `owner_candidates` identifies the planned product checkpoints; Commit 3 assigns
  exact field-level and execution ownership. Nothing is advertised as implemented.

## Commands

Offline evidence verification and regressions:

```sh
python3 scripts/check_scaleway_inventory.py
python3 scripts/test-scaleway-inventory.py
cargo test --locked --manifest-path tools/prepared-coverage-check/Cargo.toml --bin source-yaml-json
```

Exact live-source comparison (public documentation only):

```sh
python3 scripts/check_scaleway_inventory.py --fetch
```

Checkpoint completion gate:

```sh
python3 scripts/check_scaleway_inventory.py --qualify
```

The checkpoint gate accepts only the exact follow-up allowance in
[followups.json](../provider-drift/scaleway/followups.json), bound to the inventory
SHA-256 and the complete ordered candidate list. Changes to either require renewed
review. No allowance changes an unresolved entry into a supported or excluded one.
CI runs this checkpoint check and its regressions; pentest and GitHub acceptance
remain separate requirements.

Final scope qualification, mandatory by Commit 76 and again before release:

```sh
python3 scripts/check_scaleway_inventory.py --release
```

This stricter command rejects every unresolved candidate even when checkpoint
follow-up permission exists. It is expected to fail until the decisions below
are resolved; it is not a claim that the current implementation is releasable.

To stage a fresh snapshot, use `--capture --directory <new-directory>`.
Capture refuses an existing directory; it never overwrites accepted evidence.
A failed capture can leave partial raw sources but never a completed inventory
lock. Use another directory to retry, and review before replacing evidence.
Commit 4 will add semantic refresh reports; the current live comparison is exact
and may flag documentation changes unrelated to API behavior.

## Retrieval And Parsing Boundaries

Only reviewed HTTPS documentation, asset, pinned GitHub tree and pinned SDK
source paths are admitted. Redirects are rejected, proxies are disabled, and
retrieval sends no authorization or cookies. Each source is limited to 10 MiB;
the run allows at most 256 sources and 128 MiB in aggregate. Workers run
sequentially with a 60-second whole-fetch deadline and a 30-minute overall
retrieval budget. Timeout handling kills and reaps the worker, including during
stalled DNS, TLS or buffered reads. No fetched code is executed.

Index and catalog discovery each run in a killable subprocess with a 30-second
deadline. Registry scanning advances forward through bounded blocks (1 MiB per
block and 1,024 characters for the route terminator search), rejecting incomplete
or nested registry markers rather than repeatedly scanning the remaining bundle.

The already-admitted isolated Rust YAML parser rejects expansion, duplicate and
merge keys, explicit tags, excessive depth/events, multiple documents and
non-finite YAML values. Numeric scalars must use JSON number syntax; their original
lexemes are emitted without conversion through `i64` or `f64`. Python preserves
integers exactly and uses `Decimal` for decimal/exponent values. YAML-only numeric
spellings are rejected rather than rounded or silently turned into strings.
Parsing has a separate 30-second subprocess deadline. Schema
references must resolve inside the same document; no external references are
fetched. Every generated inventory is rebuilt and compared from raw sources.

Operation paths follow the runtime canonical origin-path policy with an
8,192-byte limit, extended only for `{parameter_name}` placeholders. Authority,
query/fragment delimiters, dot segments, doubled slashes, invalid template syntax,
non-ASCII literals and non-canonical percent encodings fail before admission.
This is source validation, not permission to execute an operation.

These files are public-source evidence, not runtime dependencies or credentials.
The capture uses no PyYAML dependency and changes no published crate graph.

## Approved Follow-Ups

The provider's catalog explicitly labels the following non-test schemas
`unlisted`. That is evidence of catalog visibility, not proof of either supported
public use or private-only access. Public reachability alone cannot resolve it:

- Reseller v1.
- VPC v2 Private Network with NICs and Routes with Next Hop.
- Managed PostgreSQL/MySQL v1 Encryption.
- Generative APIs v1 Consumption Limits.
- Domain v2beta1 Suggestion.
- Web Hosting v1 Control Panel and Dedibox.
- IAM v1alpha1 Unauthenticated.
- Account v3 ISO Code, Organization, Unauthenticated, Unauthenticated User and User.

The explicitly named Fake testing service is excluded as private/testing rather
than treated as a usable customer API.

Six SDK versions have no matching documentation catalog entry after normalizing
reviewed Go-package aliases: Autoscaling v1alpha1, Billing v2, DocumentDB v1beta1,
IPAM v1alpha1, Search v1alpha1, and Public Gateway v1. An older version number or
absence from navigation does not itself prove supersession. Billing v2 contains
budgets and electronic-address functionality, while the catalog exposes Billing
v2beta1 and FinOps; neither can silently stand in for the other.

The maintainer explicitly approved following these up at the end of the build,
or earlier when information arrives. Before final scope qualification, obtain
current official documentation or a recorded
Scaleway support clarification for these 20 candidates: are they supported
public customer contracts, superseded (with which successor), or private/test
interfaces? Supported contracts need authoritative source coverage and owners;
excluded contracts need explicit evidence-backed reasons. Resolve by Commit 76
and add any required implementation checkpoints before final release review.
Do not change `unresolved` merely to make the release gate green. The approved
allowance permits proceeding after the normal checkpoint review; it does not
authorize omitting a public API from the final provider.

## Credentials And Cost Policy

The maintainer has made local Scaleway credentials available, but Commit 1 does
not read them or make authenticated service calls. Credentials are never source
evidence. Public schema retrieval creates no billable cloud resources.

Later live work must agree on an isolated project, explicit resource/region and
spending limits, allowed operations, and cleanup verification before creating or
changing billable resources. Credential availability is not permission to spend.

## GitHub And Review

Local verification passed on 2026-09-26:

- Full `scripts/checks.sh` (including packaging, feature checks, Clippy,
  doctests, and default/all-feature workspace tests).
- Offline inventory reconstruction and exact live comparison of all 94 sources.
- 27 Python regression tests, including actual fetch/discovery worker timeout
  termination, 100,000 unterminated registry markers, exact numeric preservation
  across the complete parser bridge, and canonical operation-path admission.
- Four Rust parser test groups on the development toolchain and Rust 1.92.0.
- Isolated tool Clippy with warnings denied and formatting checks.
- SBOM freshness/completeness, documentation links, workflow governance,
  release-plan regressions, whitespace, and 500-line code-file policy.
- Checkpoint qualification with the exact approved follow-ups; final release
  qualification was confirmed to reject the unresolved entries.

No published manifest or lockfile changed. The isolated tooling SBOM now includes
the additional parser binary. This verification is not a pentest result.

The initial Commit 1 pentest reported registry-scan complexity, numeric rounding,
and unsafe path admission. The remediations above preserve the accepted inventory
and its follow-up digest unchanged. Independent remediation retest remains pending.

CodeQL Default Setup was verified through GitHub's API as configured for Rust,
Python and Actions. No advanced workflow is introduced. GitHub documents that
default setup scans pushes to default/protected branches and pull requests
targeting those branches. See [GitHub setup types](https://docs.github.com/en/code-security/concepts/code-scanning/setup-types).

After publishing the branch, protect `scaleway` so direct pushes trigger Default
Setup, or keep a pull request from `scaleway` to `main` open without merging it.
Verify an actual successful CodeQL run on the checkpoint's final revision; merely
having the configuration enabled or Rust CI green is not sufficient.

The first complete review range is `v1.1.0..HEAD`, including the intervening
planning changes. The branch setup hash is
`49ee09d7284b187d4b1d4b032de20765a6b3e524`. No pentest result is claimed yet.
