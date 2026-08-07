# cloud-sdk 0.62.0 Milestone Notes

Status: implementation stop reached; pentest required.

Release date: 2026-08-07

Security-Review: PENDING
Pentest: PENDING
Publication: DEFERRED TO v0.65.0

## Overview

v0.62 freezes the provider-neutral contracts after the complete unpublished
OVHcloud probe, the unchanged Robot wire fixture, and source-complete Hetzner
vertical slices. It remains an internal tag and publishes no crate.

## Hetzner Vertical Slices

- Added all source fields for paginated `list_locations`.
- Reused the complete action model for `poweron_server`.
- Retained `get_zone_zonefile` output in protected multiline storage.
- Added all `get_certificate` fields with protected PEM and error text.
- Added all nested `list_storage_boxes` fields and bounded incremental JSON
  admission before duplicate-rejecting protected model decoding.
- Reused the checked typed API error and exact `delete_certificate` 204 path.
- Corrected generated response bindings to preserve Cloud, DNS, security, and
  Storage service identities independently of the two upstream API documents.
- Corrected the Storage management API association to bearer authentication,
  derived generation policy from the reviewed provider lock, and renamed the
  misleading DNS drift identities to Storage identities.
- Required every numbered response array to contain no more records than its
  validated `per_page` value before typed allocation begins.
- Made protected JSON DOM and typed response allocations fallible and exposed
  allocation failure through payload-free model errors.
- Removed infallible `Arc` and `Box` allocation from protected response text
  and quota metadata; secret-bearing response models are intentionally not
  infallibly cloneable.
- Added `HetznerQuota::to_quota_buckets` so the compact inline provider bucket
  enters neutral `decide_delay` policy without bespoke retry logic or heap
  allocation.
- Sorted bounded JSON object fields once, rejected adjacent duplicates, used
  binary field lookup, and lowered the per-object ceiling to 256 fields.
- Added `decode_associated_checked_response` so blocking, Send-async, and local
  async execution results retain an unforgeable `AssociatedCheckedResponse<O>`
  marker through typed decoding without reopening raw response bytes.

## Neutral Freeze

No probe required a provider enum, runtime, executor, retry engine, parser,
credential store, filesystem, clock, or transport exception in `cloud-sdk`.
The public review records the accepted contracts and rejected abstractions.

Testkit now admits exact successful `2xx` statuses and classifies all in-memory
mock failures as proven `NotSent`, allowing permit-authorized `201` and `204`
fixtures without pretending peer I/O occurred.

The vertical gate now uses operation-valid Location, Certificate, Zonefile,
and Storage Box fixtures, checks Storage bearer authentication, typed-decodes
all three executor results, and proves the generic `{"ok":true}` envelope is
rejected for every source-complete read slice.

## Versions

| Crate | Source version | Cumulative change | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.62.0` | code | deferred to v0.65.0 |
| `cloud-sdk-hetzner` | `0.39.1` | code | deferred |
| `cloud-sdk-reqwest` | `0.33.0` | dependency | deferred |
| `cloud-sdk-sanitization` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.29.1` | code | deferred |

## Release Gate

Run `scripts/release_0_62_gate.sh` on the clean evidence commit after the
incremental pentest and final retest. GitHub CI and CodeQL must be green on the
unchanged commit before the signed internal tag. Do not publish crates.
