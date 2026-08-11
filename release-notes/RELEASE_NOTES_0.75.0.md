# cloud-sdk 0.75.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-11

Security-Review: PASS
Pentest: PASS
Publication: PENDING

## Overview

v0.75 introduces the first Hetzner Robot runtime primitive: bounded atomic
`application/x-www-form-urlencoded` encoding. It also closes the cumulative
v0.71-v0.75 publication train containing complete named DNS, Security, and
Console Storage Box clients, the Robot operation source lock, and retirement
of the experimental FIPS transport.

## Robot Form Codec

- Added ordered borrowed form fields with explicit public or sensitive value
  classification and payload-free diagnostics.
- Preserved duplicate names for Robot array parameters such as `server[]` and
  indexed firewall names such as `rules[input][0][src_ip]`.
- Require one nonempty parameter root followed only by complete bracketed
  components, rejecting malformed nested names before encoding.
- Added standard form encoding: spaces become `+`; literal separators,
  controls, brackets, `+`, `~`, and non-ASCII UTF-8 bytes are percent encoded
  with uppercase hexadecimal digits.
- Added exact checked preflight, field and aggregate caps, immutable snapshot
  replay, and unchanged output for validation or capacity rejection.
- Clear the complete admitted output before writing and again when the encoded
  body guard is dropped, preventing stale secret tails after buffer reuse.
- Added exact wire-fixture, every-capacity, all-ASCII, UTF-8, exact-bound,
  redaction, cleanup, and dedicated fuzz coverage.

## Cumulative Checkpoint

- Includes named blocking, Send-async, and local-async workflows for all 24
  DNS, 14 Security, and 31 Console Storage Box operations from v0.71-v0.73.
- Includes the complete v0.74 Robot lock: 105 headings, 89 active operations,
  and 16 excluded deprecated legacy Storage Box operations.
- Removes the experimental AWS-LC FIPS transport and defers future FIPS work
  until Brynja completes its separate qualification.
- Keeps Robot credentials, endpoint operations, error decoding, retries,
  clients, and live execution assigned to later milestones.

## Versions

| Crate | Previous published | v0.75 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.70.0` | `0.75.0` | yes, cumulative code |
| `cloud-sdk-hetzner` | `0.41.0` | `0.42.0` | yes, cumulative code |
| `cloud-sdk-reqwest` | `0.34.1` | `0.35.0` | yes, cumulative code |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | no, unchanged |
| `cloud-sdk-testkit` | `0.30.1` | `0.30.2` | yes, dependency-only |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW_0.75.0.md`](../docs/PUBLIC_API_REVIEW_0.75.0.md)
- [`docs/DEPENDENCY_REVIEW_0.75.0.md`](../docs/DEPENDENCY_REVIEW_0.75.0.md)
- [`docs/THREAT_MODEL_DELTA_0.75.0.md`](../docs/THREAT_MODEL_DELTA_0.75.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.75.0.md`](../docs/REJECTED_ABSTRACTIONS_0.75.0.md)
- [`docs/MIGRATION_0.75.0.md`](../docs/MIGRATION_0.75.0.md)
- [`security/pentest/v0.75.0.md`](../security/pentest/v0.75.0.md)

## Release Gate

Run `scripts/release_0_75_gate.sh` on the clean final evidence commit. GitHub
CI and CodeQL must be green on that unchanged commit before tagging and
publishing the selected crates.
