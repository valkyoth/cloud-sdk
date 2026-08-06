# cloud-sdk 0.57.0 Milestone Notes

Status: release candidate; incremental pentest and final retest passed.

Release date: 2026-08-06

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.60.0

## Overview

v0.57 source-locks an unpublished OVHcloud API v2 architecture probe against
four official documents and eight read-only production IAM operations. The
probe supplies evidence for the neutral-contract conformance work planned in
v0.58-v0.61; it is not a provider crate or an OVHcloud support claim.

This milestone receives an incremental pentest from signed v0.56.0, the full
local and GitHub gates, and a normal signed tag. No crate is published until
the v0.60.0 checkpoint.

## Source Lock

- Locked the official API v2 index, IAM console schema, API v2 guide, and
  service-account OAuth guide by exact URL, byte length, and SHA-256.
- Pinned both GitHub-hosted guides to official documentation commit
  `eb5d926b9030000cfb03386c4cbe6d60491ab63a`.
- Derived authority, token endpoint, OAuth expiry, schema-version, cursor,
  task, event, operation, response-model, and stability evidence from the
  authenticated source bytes.
- Selected eight production `GET` IAM operations spanning collections and
  resource identities, including four cursor-paginated collection routes.
- Added deterministic canonical lock and observation documents through the
  v0.56 provider-neutral drift engine.

## Security Boundaries

- Rejects source substitution, duplicate JSON members, unexpected authority,
  missing authentication/task evidence, non-production operations, and methods
  other than `GET`.
- Keeps all retrieval credential-free, exact-origin, redirect-free, bounded,
  and authenticated before provider observation construction.
- Resolves each source once and connects the TLS socket only to the validated
  global address set while retaining original-host SNI, certificate, and
  `Host` verification; duplicate answers are removed, at most eight unique
  destinations are admitted, all setup attempts share one ten-second deadline,
  and successful TLS setup restores the normal HTTP I/O timeout.
- Authenticates both observed IAM wire orderings by sorting only the unique
  top-level path set before hashing every value and remaining list order in the
  strict UTF-8 JSON object; duplicate keys and non-finite constants fail closed.
- Records regional API and token authorities distinctly; no credential is
  created, loaded, sent, or accepted by the probe.
- Excludes the probe from Cargo workspace membership, package discovery,
  release planning, publish order, and supported-provider claims.

## Versions

| Crate | Source version | Publication |
| --- | --- | --- |
| `cloud-sdk` | `0.57.0` | deferred to v0.60.0 |
| `cloud-sdk-hetzner` | `0.39.0` | unchanged |
| `cloud-sdk-reqwest` | `0.32.4` | unchanged |
| `cloud-sdk-sanitization` | `0.18.0` | unchanged |
| `cloud-sdk-testkit` | `0.29.0` | unchanged |

## Documentation

- [`provider-probes/ovhcloud-v2/README.md`](../provider-probes/ovhcloud-v2/README.md)
- [`provider-probes/ovhcloud-v2/THREAT_MODEL.md`](../provider-probes/ovhcloud-v2/THREAT_MODEL.md)
- [`docs/MIGRATION_0.57.0.md`](../docs/MIGRATION_0.57.0.md)
- [`docs/PUBLIC_API_REVIEW_0.57.0.md`](../docs/PUBLIC_API_REVIEW_0.57.0.md)
- [`docs/DEPENDENCY_REVIEW_0.57.0.md`](../docs/DEPENDENCY_REVIEW_0.57.0.md)

## Release Gate

The incremental pentest and final retest passed, with permanent evidence at
[`security/pentest/v0.57.0.md`](../security/pentest/v0.57.0.md). Tag only after
`scripts/release_0_57_gate.sh` passes on the clean evidence commit and GitHub CI
and CodeQL are green. Do not publish crates for this internal milestone.
