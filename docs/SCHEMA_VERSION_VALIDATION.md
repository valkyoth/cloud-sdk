# Schema Version Validation

Provider schema versions are compatibility inputs, not ordinary transport
metadata. `cloud-sdk::schema` models them without allocation, network access,
or an ambient provider default.

## Reviewed Evidence

`SchemaVersion` accepts only canonical nonzero-major `major.minor` decimal
text. `ReviewedSchemaMajor` binds the admitted major to the exact SHA-256 of a
reviewed provider source. Callers must rotate that evidence through normal
source-lock review when a provider introduces or retires a major line.

The digest proves which public source was reviewed. It is not a credential,
signature, password hash, or substitute for authenticated source fetching.

## Validation-Only Overrides

`ValidationSchemaHeader` can be constructed only when the selected version
matches the reviewed major. Its only encoder is deliberately named
`with_validation_header`; it creates a bounded public request header inside a
closure and clears caller scratch on every path.

Do not add this header automatically to normal production requests unless the
provider explicitly defines that behavior. OVHcloud documents
`X-Schemas-Version` for validation before an account-level major migration and
uses the account-selected major when the header is absent. The unpublished
probe therefore source-locks `1.0` as reviewed test evidence while leaving the
header absent by default.

## Drift Policy

- Reject malformed, zero, overflowing, or noncanonical versions.
- Reject a major that differs from `ReviewedSchemaMajor`.
- Never infer compatibility from a minor number or silently select the newest
  major.
- Keep account configuration, provider availability windows, and migration
  decisions outside the SDK core.
- Update source evidence, conformance fixtures, migration notes, and pentest
  coverage before admitting a new major.
