# v0.66.0 Public API Review

Status: implementation stop reached; pentest required.

Scope: changes from signed v0.65.0 through v0.66.0.

## Added Provider API

- `SecurityResource` and `SecurityResourceKind` identify complete certificate
  and SSH-key response families.
- `SshKey` exposes validated metadata and closure-scoped inspection of its
  protected, structurally decoded OpenSSH public key. Its
  `sha256_fingerprint` accessor exposes an SDK-computed 32-byte identity value;
  `fingerprint` remains the provider's verified legacy MD5 text.
- `CertificateIssuanceState` and `CertificateRenewalState` replace unchecked
  provider-state strings.
- `Certificate`, `CertificateStatus`, `CertificateError`, and
  `CertificateUse` expose private fields through read-only accessors.
- `CertificateError::code_text` preserves certificate-specific provider codes
  even when `CertificateError::code` classifies them as `ApiErrorCode::Unknown`.
- `CompositeResult::security_resource` returns a source-complete certificate
  created by `create_certificate`.

## Changed Provider API

- Certificate singleton results now use
  `HetznerSuccess::SecurityResource(SecurityResource::Certificate(...))`.
- Certificate fields are no longer public and secret-bearing aggregates no
  longer implement ordinary equality.
- Certificate timestamps use validated `UtcTimestamp` values and managed
  states use closed source-known enums.

## Compatibility

These are intentional pre-1.0 provider API changes. No provider-neutral API,
default feature, transport, runtime, TLS, filesystem, clock, or secret-store
boundary changes. The models remain behind the existing optional
`cloud-sdk-hetzner/serde` feature.
