# v0.66.0 Dependency Review

Status: implementation stop reached; pentest required.

The existing optional `cloud-sdk-hetzner/serde` feature admits `ssh-key 0.6.7`
with only `alloc` and `md-5 0.11.0` with defaults disabled. They structurally
decode provider-returned OpenSSH public keys, bind Hetzner's legacy MD5
fingerprint to the RFC 4253 key bytes, and derive a SHA-256 fingerprint. Neither
dependency enters the provider's default graph or enables `std`.

MD5 is not collision-resistant and is not admitted for signatures,
authorization, or any new protocol. Its only use is checking the legacy value
returned by Hetzner against the same public key in that response. The SDK's
SHA-256 result is the identity value exposed for caller comparisons. The full
admission record is
[`dependency-admission-ssh-key.md`](dependency-admission-ssh-key.md).

The source version of `cloud-sdk-hetzner` remains at its published 0.40.0
package number until v0.70.0. No crate is selected for publication.

## Root Lockfile Changes

| Package | Previous | v0.66 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.65.0` | `0.66.0` | Workspace milestone metadata; no third-party edge changed. |
| `ovhcloud-v2-probe` | `0.65.0` | `0.66.0` | Inherits workspace metadata; source and dependency graph are unchanged. |
| `base64ct` | `-` | `1.8.3` | Transitive constant-time Base64 implementation used by `ssh-encoding`; `no_std`. |
| `block-buffer` | `-` | `0.10.4` | Transitive SHA-256 buffering for `ssh-key`. |
| `block-buffer` | `-` | `0.12.1` | Transitive MD5 buffering for `md-5`. |
| `cipher` | `-` | `0.4.4` | Mandatory type-level dependency of `ssh-cipher`; no encryption algorithm feature is enabled. |
| `cpufeatures` | `-` | `0.2.17` | Transitive SHA-256 backend selection. |
| `crypto-common` | `-` | `0.1.7` | Transitive digest traits for SHA-256. |
| `crypto-common` | `-` | `0.2.2` | Transitive digest traits for MD5. |
| `digest` | `-` | `0.10.7` | Transitive digest API used by `sha2`. |
| `digest` | `-` | `0.11.3` | Transitive digest API used by `md-5`. |
| `generic-array` | `-` | `0.14.7` | MSRV-compatible transitive fixed-array support for the SHA-256 branch. |
| `hybrid-array` | `-` | `0.4.14` | Transitive fixed-array support for the MD5 branch. |
| `inout` | `-` | `0.1.4` | Transitive buffer traits used by the inert `ssh-cipher` layer. |
| `md-5` | `-` | `0.11.0` | Direct optional legacy-fingerprint verifier; defaults disabled. |
| `pem-rfc7468` | `-` | `0.7.0` | Transitive text encapsulation parser used by `ssh-encoding`. |
| `sha2` | `-` | `0.10.9` | Transitive SHA-256 implementation used by `ssh-key`. |
| `signature` | `-` | `2.2.0` | Mandatory `ssh-key` public-key type dependency; only `alloc` is activated. |
| `ssh-cipher` | `-` | `0.2.0` | Mandatory `ssh-key` dependency; no cipher implementation feature is activated. |
| `ssh-encoding` | `-` | `0.2.0` | OpenSSH Base64 and RFC 4253 encoding/decoding support. |
| `ssh-key` | `-` | `0.6.7` | Direct optional maintained OpenSSH/RFC 4253 public-key parser; defaults disabled, `alloc` only. |
| `typenum` | `-` | `1.20.1` | Transitive compile-time digest-size support. |
| `version_check` | `-` | `0.9.5` | Transitive build dependency of `generic-array`. |

The excluded fuzz package receives the same locked response-model graph so its
named SSH-key seed exercises the production parser. Both lockfiles contain the
same exact additions.
