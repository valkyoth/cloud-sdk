# v0.66.0 Dependency Review

Status: release candidate; pentest and final retest passed.

The optional `cloud-sdk-hetzner/serde` feature reuses exact first-party
`base64-ng 2.0.1` and admits `md-5 0.11.0` plus `sha2 0.10.9`, all with defaults
disabled. A bounded first-party parser validates provider-returned OpenSSH text
and RFC 4253 key structures, binds Hetzner's legacy MD5 fingerprint to the wire
key, and derives a SHA-256 identity fingerprint. None enters the provider's
default graph or enables `std`.

MD5 is not collision-resistant and is not admitted for signatures,
authorization, or any new protocol. Its only use is checking the legacy value
returned by Hetzner against the same public key in that response. The SDK's
SHA-256 result is the identity value exposed for caller comparisons. The full
admission record is
[`dependency-admission-ssh-public-key.md`](dependency-admission-ssh-public-key.md).

The source version of `cloud-sdk-hetzner` remains at its published 0.40.0
package number until v0.70.0. No crate is selected for publication.

## Root Lockfile Changes

| Package | Previous | v0.66 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.65.0` | `0.66.0` | Workspace milestone metadata; no third-party edge changed. |
| `ovhcloud-v2-probe` | `0.65.0` | `0.66.0` | Inherits workspace metadata; source and dependency graph are unchanged. |
| `block-buffer` | `-` | `0.10.4` | Transitive SHA-256 buffering for direct `sha2`. |
| `block-buffer` | `-` | `0.12.1` | Transitive MD5 buffering for `md-5`. |
| `cpufeatures` | `-` | `0.2.17` | Transitive SHA-256 backend selection. |
| `crypto-common` | `-` | `0.1.7` | Transitive digest traits for SHA-256. |
| `crypto-common` | `-` | `0.2.2` | Transitive digest traits for MD5. |
| `digest` | `-` | `0.10.7` | Transitive digest API used by `sha2`. |
| `digest` | `-` | `0.11.3` | Transitive digest API used by `md-5`. |
| `generic-array` | `-` | `0.14.7` | Fixed-array support for the SHA-256 branch. |
| `hybrid-array` | `-` | `0.4.14` | Fixed-array support for the MD5 branch. |
| `md-5` | `-` | `0.11.0` | Direct optional legacy-fingerprint verifier; defaults disabled. |
| `sha2` | `-` | `0.10.9` | Direct optional SHA-256 identity derivation; defaults disabled. |
| `typenum` | `-` | `1.20.1` | Transitive compile-time digest-size support. |
| `version_check` | `-` | `0.9.5` | Transitive build dependency of `generic-array`. |

`base64-ng 2.0.1` was already locked and reviewed in v0.65; v0.66 adds it to
the provider Serde feature without changing its version. The excluded fuzz
package receives the same locked response-model graph so its named SSH-key
seed exercises the production parser. Both lockfiles must contain the same
exact digest additions and no `ssh-key` package.
