# v0.61.0 Dependency Review

Status: implementation review complete; pentest and final release gate pending.

Scope: dependency changes from published v0.60.0 through v0.61.0.

## Decision

No third-party dependency or feature version is added or changed. The
nonpublishable OVHcloud harness uses only existing workspace packages:

- `cloud-sdk` and `cloud-sdk-testkit` in its default credential-free graph;
- optional `cloud-sdk-reqwest` blocking rustls transport for the ignored live
  smoke;
- optional `cloud-sdk-sanitization` for complete token-source cleanup.

The default harness graph remains transport-free. The live feature reuses the
already admitted, pinned reqwest/rustls/AWS-LC graph and introduces no new
runtime, parser, secret store, clock, or retry implementation.

## Controls

The harness is a workspace member, so root Cargo lock, clippy, tests, deny,
audit, SBOM, platform, and dependency-boundary checks cover it. Cargo metadata
must report `publish = false`, and release-plan and publisher checks reject all
OVHcloud entries. No crate is selected for crates.io at v0.61.0.

## Root Lockfile Changes

| Package | Previous | v0.61 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.60.0` | `0.61.0` | Source milestone metadata; no public API or third-party graph change. |
| `ovhcloud-v2-probe` | `-` | `0.61.0` | Exact nonpublishable workspace harness using only reviewed workspace dependencies. |
