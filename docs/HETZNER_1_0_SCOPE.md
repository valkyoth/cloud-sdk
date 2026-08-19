# Hetzner 1.0 Scope

Status: finite source-locked scope for the `cloud-sdk-hetzner` 1.0 release.

## Included Surface

| Surface | Active | Deprecated or removed | Source lock |
| --- | ---: | ---: | --- |
| Cloud, DNS, Security, and Console Storage OpenAPI | 208 | 13 | [`API_FINGERPRINTS.tsv`](API_FINGERPRINTS.tsv) |
| Robot Webservice | 89 | 16 | [`ROBOT_WIRE_SOURCE_LOCK.md`](ROBOT_WIRE_SOURCE_LOCK.md) |
| Canonical Server Metadata | 7 | removed aliases tracked separately | [`METADATA_FINGERPRINTS.tsv`](METADATA_FINGERPRINTS.tsv) |
| **Total executable 1.0 scope** | **304** | **29** | all three locks |

Every active operation must have an exact request contract, bounded response
contract, authentication and endpoint authority, cleanup behavior, and
blocking, `Send` async, and local-async execution evidence where transport is
applicable. Deprecated operations are retained only in source evidence and are
not added to the public executable surface.

The OpenAPI lock contains 221 operations in total. The Robot reference lock
contains 105 operation headings in total. Server Metadata is prose-only and is
therefore locked separately from both machine-readable API documents.

## Server Metadata

The supported metadata surface is exactly:

- `GET http://169.254.169.254/hetzner/v1/metadata`
- `GET http://169.254.169.254/hetzner/v1/metadata/hostname`
- `GET http://169.254.169.254/hetzner/v1/metadata/instance-id`
- `GET http://169.254.169.254/hetzner/v1/metadata/public-ipv4`
- `GET http://169.254.169.254/hetzner/v1/metadata/private-networks`
- `GET http://169.254.169.254/hetzner/v1/metadata/availability-zone`
- `GET http://169.254.169.254/hetzner/v1/metadata/region`

These routes are credential-free and confined to the exact IPv4 link-local
HTTP destination. The SDK exposes no custom metadata destination, credential
attachment, redirect following, proxy selection, retry, mutation, ambient
environment discovery, or TLS claim. Summary, scalar, and private-network YAML
responses are strictly bounded and source-locked.

The legacy EC2-compatible aliases removed on 2026-08-01 are deliberately
unsupported: `/2009-04-04/meta-data`, its key route, `/user-data`,
`/latest/meta-data`, its key route, and `/latest/user-data`.

## Explicit Exclusions

Hetzner Object Storage uses the standard S3 API without a reviewed
Hetzner-specific extension. It is outside this provider's 1.0 surface; callers
should select a separately reviewed S3 implementation.

Robot domain registration is unavailable through the Robot Webservice. The
official documentation directs that workflow to a separate mail interface, so
it cannot be represented as an SDK operation.

Future additive provider APIs are not silently covered by this statement.
They must first appear in source-drift evidence, receive models and execution
contracts, and pass the normal security and release process.

## Maintenance

Run the complete upstream check before endpoint work and every release:

```bash
scripts/check_hetzner_api_surface.sh --fetch
```

The command checks both OpenAPI documents, operation and schema fingerprints,
the Robot operation and wire locks, the canonical metadata prose lock, and the
Hetzner changelog RSS lock. Additions, removals, deprecations, parameter or
schema changes, metadata route/format changes, and reviewed changelog semantic
changes fail closed until their locks and executable contracts are updated
together. See [`API_DRIFT_MAINTENANCE.md`](API_DRIFT_MAINTENANCE.md).
