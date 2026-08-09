# SSH Public-Key Validation Dependency Admission

Status: admitted only behind the non-default `cloud-sdk-hetzner/serde` feature.

Checked: 2026-08-09.

## Decision

Provider-returned SSH keys are decoded as strict RFC 4648 Base64 and validated
as bounded RFC 4253 wire structures before the checked model can claim that
they are valid. A small first-party parser performs this validation directly
inside cleanup-owned storage. It shares one exact algorithm identity enum with
the request model and creates no second owned public-key representation.

| Package | Version | Direct features | License |
| --- | --- | --- | --- |
| `base64-ng` | `2.0.1` | defaults disabled | MIT OR Apache-2.0 |
| `md-5` | `0.11.0` | defaults disabled | MIT OR Apache-2.0 |
| `sha2` | `0.11.0` | defaults disabled | MIT OR Apache-2.0 |

All three packages are `no_std` and pinned exactly. `base64-ng` is an existing
first-party dependency already admitted for bounded Basic authentication. The
two RustCrypto digest crates add no filesystem, network, TLS, process, random
number, native-code, or operating-system capability.

## Feature And Graph Policy

The packages enter only the provider Serde graph. The default provider graph
remains allocation-free, transport-free, and independent of them.

`sha2 0.11.0` and `md-5 0.11.0` share RustCrypto digest 0.11. The v0.70 update
therefore removes the old digest 0.10 duplicate graph and its Cargo Deny
exceptions without changing runtime capability.

`scripts/check_security_response_models.sh` proves exact graph membership,
default-graph isolation, complete algorithm fixtures, and the absence of
`ssh-key` and `zeroize`. `scripts/check_sanitization_boundary.sh` rejects any
provider path that bypasses `cloud-sdk-sanitization`. Cargo Deny, Cargo Audit,
the MSRV matrix, `no_std` checks, both lockfiles, and generated SPDX SBOMs cover
the admitted graph.

## Security Boundary

The parser accepts exactly these text and wire algorithm identifiers:

- `ssh-ed25519`
- `ssh-rsa`
- `ecdsa-sha2-nistp256`
- `ecdsa-sha2-nistp384`
- `ecdsa-sha2-nistp521`
- `sk-ssh-ed25519@openssh.com`
- `sk-ecdsa-sha2-nistp256@openssh.com`

It rejects unknown, prefixed, and vendor-suffixed identifiers; noncanonical
Base64; truncated or trailing wire data; text/wire algorithm disagreement;
noncanonical RSA positive integers; wrong Ed25519 lengths; wrong ECDSA curve
identifiers or SEC1 point shapes; and malformed FIDO application strings.
Validation is structural and does not prove that an ECDSA point lies on its
named curve. These are public authentication identifiers, not private keys or
signature inputs.

Decoded RFC 4253 bytes live in one bounded allocation guarded by
`cloud-sdk-sanitization::SecretBuffer`, which clears them on every success and
error exit. The protected OpenSSH source string is transferred directly from
the checked JSON parser into `SensitiveText`; the wire parser creates no owned
copy of that text.

MD5 is cryptographically broken. It is used only to detect inconsistency
between two fields in one provider response. It is not used as proof against a
malicious collision, for authorization, or by any request-signing operation.
The model separately exposes an SDK-computed 32-byte SHA-256 fingerprint for
identity and audit workflows.

## Rejected Alternative

RustCrypto `ssh-key 0.6.7` was evaluated during v0.66 but not admitted. Its
feature-specific ECDSA variants made the accepted algorithm policy easy to
misconfigure, its extensible `Other` variant did not enforce this SDK's closed
allowlist, and parsing created an owned public-key/comment model without the
project's required cleanup semantics. Enabling more dependency features would
fix valid ECDSA rejection but not the ownership or exact-policy mismatch.
