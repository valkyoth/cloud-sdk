# SSH Public-Key Dependency Admission

Status: admitted only behind the non-default `cloud-sdk-hetzner/serde` feature.

Checked: 2026-08-08.

## Decision

Provider-returned SSH keys must be decoded as OpenSSH text and RFC 4253 wire
data before the checked model can claim that they are valid. The maintained
RustCrypto `ssh-key` parser is preferred over a second in-repository Base64 and
SSH binary parser. `md-5` is admitted only to verify Hetzner's documented
legacy fingerprint field against the decoded public key.

| Package | Version | Direct features | License | MSRV |
| --- | --- | --- | --- | --- |
| `ssh-key` | `0.6.7` | defaults disabled; `alloc` | Apache-2.0 OR MIT | 1.65 |
| `md-5` | `0.11.0` | defaults disabled | MIT OR Apache-2.0 | 1.85 |

Both packages are `no_std`. Their versions were the latest stable releases on
the review date; the newer `ssh-key 0.7.0-rc.11` line was a prerelease and was
not admitted. The workspace pins both direct versions exactly.

## Feature And Graph Policy

`ssh-key` receives only `alloc`, which is required for OpenSSH comments and
fallible RFC 4253 serialization. Its default `std`, ECDSA, and random-number
features are disabled. No private-key, signing, encryption, RNG, filesystem,
network, TLS, process, or native-code capability is enabled. `md-5` has no
feature enabled.

The transitive graph includes Base64/PEM and SSH encoding types, SHA-256,
digest support, and inactive SSH cipher abstractions. No concrete cipher
feature is active. The existing `subtle` and transitive `zeroize` packages are
reused. Exact additions are recorded in
[`DEPENDENCY_REVIEW_0.66.0.md`](DEPENDENCY_REVIEW_0.66.0.md).

Stable `ssh-key 0.6.7` uses the RustCrypto digest 0.10 line, while latest stable
`md-5 0.11.0` uses digest 0.11. Cargo Deny therefore carries exact temporary
exceptions for `digest 0.10.7`, `block-buffer 0.10.4`, and
`crypto-common 0.1.7`. Downgrading the direct MD5 package would violate the
latest-stable dependency policy without reducing runtime capability. The
exceptions should be removed when a stable `ssh-key` release converges on the
new digest line.

`scripts/check_security_response_models.sh` proves that neither direct package
enters the default provider graph and that both exact versions enter the Serde
graph. Cargo Deny, Cargo Audit, the MSRV matrix, `no_std` checks, both lockfiles,
and generated SPDX SBOMs cover the admitted graph.

`ssh-key` internally depends on `zeroize 1.9.0`; first-party response storage
and temporary RFC 4253 bytes still use only `cloud-sdk-sanitization` for their
cleanup contract. `scripts/check_sanitization_boundary.sh` allows the exact
`zeroize -> ssh-key -> cloud-sdk-hetzner` transitive path and rejects any wider
provider path or direct substitution of the first-party cleanup primitive.

## Security Boundary

The provider response parser first applies the SDK's bounded request-side
algorithm policy, then asks `ssh-key` to decode the complete OpenSSH and RFC
4253 structure. It serializes the parsed public key to bounded wire bytes,
computes the Hetzner compatibility fingerprint, and clears the temporary bytes
through `cloud-sdk-sanitization` on every exit.

MD5 is cryptographically broken. It is used only to detect inconsistency
between two fields in one provider response. It is not used as proof against a
malicious collision, for authorization, or by any request-signing operation.
The model separately exposes an SDK-computed 32-byte SHA-256 fingerprint for
identity and audit workflows.

## Rejected Alternatives

- Extending the lexical request validator would still duplicate Base64,
  length-prefixed SSH wire parsing, algorithm matching, and key-shape checks.
- Accepting the provider fingerprint independently leaves contradictory
  response identities available to automation.
- Treating the MD5 field as a modern security identifier would inherit known
  collision weaknesses and is explicitly outside this admission.
- Enabling `ssh-key` defaults would add unrelated crypto implementations,
  randomness, and `std` to an optional response parser.
