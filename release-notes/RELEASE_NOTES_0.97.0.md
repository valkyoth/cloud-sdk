# cloud-sdk 0.97.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-19

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.100.0

## Overview

v0.97 closes the canonical Hetzner Server Metadata surface, validates the
latest source-locked response semantics, and fixes the finite denominator for
the complete Hetzner 1.0 provider. It is an internal cumulative tag and
publishes no crate.

## Server Metadata

- Added exact bodyless GET contracts for the metadata summary and its six
  canonical child reads.
- Confined execution to `http://169.254.169.254:80/` with a provider-owned
  fixed endpoint policy and no custom destination, credential, redirect,
  proxy, retry, mutation, TLS claim, or ambient environment interpretation.
- Added strict allocation-free summary and scalar decoding plus bounded
  private-network YAML decoding with duplicate, unknown, canonical address,
  CIDR, MAC, gateway, subnet, interface, network, alias, and aggregate checks.
- Added blocking, `Send` async, and local-async one-attempt helpers with exact
  endpoint verification before executor access.
- Added separate credential-free link-local raw reqwest adapter builder types;
  HTTPS-only builders cannot carry a runtime HTTP downgrade mode.

## Source And Response Qualification

- Added a deterministic metadata prose fingerprint sourced from the official
  OpenAPI description and mutation tests for routes, fields, duplicates,
  removed aliases, and section identity.
- Integrated metadata into the complete live Hetzner source drift command and
  added a dedicated seven-route decoder fuzz target with canonical corpora and
  the full response bound.
- Added exact cross-field validation for Load Balancer health details and HTTP
  status codes and for Primary IP `assignee_type`/`assignee_id` pairs from the
  August 2026 changelog.
- Documented 208 active OpenAPI operations, 89 active Robot operations, and
  seven active metadata reads as the 304-operation 1.0 scope. The 29
  deprecated operations, standard S3 API, and mail-only Robot domain
  registration remain explicit exclusions.

## Versions

| Crate | Published | v0.97 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.95.0` | `0.97.0` | deferred |
| `cloud-sdk-hetzner` | `0.46.0` | `0.46.0` | code; deferred |
| `cloud-sdk-reqwest` | `0.36.0` | `0.36.0` | code; deferred |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.31.0` | `0.31.0` | unchanged |

## Pentest Remediation

- Reused the established server-name and public-address policies for metadata
  hostname and public IPv4 values in scalar and summary responses.
- Replaced quadratic alias duplicate scans with a fixed 512-address scratch
  array, checked range access, bounded sorting, and adjacent duplicate checks;
  item 513 is rejected before address parsing.
- Split raw HTTPS and link-local HTTP construction into distinct blocking and
  async builder types so HTTPS builders cannot carry a runtime downgrade mode.
- Restored the shared scalar policy for public IPv4 responses: exactly one
  terminal LF is accepted, while CRLF and repeated newlines remain rejected.

## Stop Gate

The incremental pentest and remediation retest are green. Run
`scripts/release_0_97_gate.sh` against the exact final evidence commit and
require green GitHub CI and CodeQL before tagging. Do not publish crates; the
cumulative public checkpoint is v0.100.0.

## Result

v0.97.0 is ready for its internal signed tag after the clean local release
gate and GitHub CI and CodeQL pass. No crate is selected for publication.
