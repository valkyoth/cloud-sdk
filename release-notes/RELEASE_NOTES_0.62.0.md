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

## Neutral Freeze

No probe required a provider enum, runtime, executor, retry engine, parser,
credential store, filesystem, clock, or transport exception in `cloud-sdk`.
The public review records the accepted contracts and rejected abstractions.

Testkit now admits exact successful `2xx` statuses and classifies all in-memory
mock failures as proven `NotSent`, allowing permit-authorized `201` and `204`
fixtures without pretending peer I/O occurred.

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
