# Dependency Review 0.87.0

Status: implementation stop; pentest required.

No external package or feature was added. Robot traffic reuses `cloud-sdk`,
`cloud-sdk-sanitization`, the provider's existing incremental JSON parser, and
the existing `alloc`/`serde` graph.

The default graphs remain free of network clients, TLS, async runtimes,
filesystem, clocks, native code, and secret stores. Traffic preparation and
decoding do not add cryptography. Existing exact dependency pins and AWS-LC
admission policy are unchanged. Crates.io publication remains deferred to
v0.90.0.

## Lockfile Changes

The cumulative internal train is compared with the latest public v0.85
checkpoint.

| Package | Previous | v0.87 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.85.0` | `0.87.0` | Advance the internal facade through reverse DNS and traffic. |
| `ovhcloud-v2-probe` | `0.85.0` | `0.87.0` | Advance the excluded workspace probe identity only. |

The exact local core requirement advances in the fuzz and reqwest
feature-unification lockfiles. Published provider, reqwest, sanitization, and
testkit identities remain unchanged. External package versions, checksums,
features, and sources are unchanged.

## Publication Selection

| Package | Published | v0.87 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.85.0` | `0.87.0` | code | no |
| `cloud-sdk-hetzner` | `0.44.0` | `0.44.0` | code | no |
| `cloud-sdk-reqwest` | `0.35.2` | `0.35.2` | unchanged | no |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.4` | `0.30.4` | unchanged | no |

`scripts/release_crates.py` must select no package for this internal milestone.
Fuzz, tools, isolated tests, and the OVHcloud probe remain excluded.
