# Migrating To v0.66.0

v0.66.0 is an internal source milestone after the published v0.65.0
checkpoint. No crate is published; applications remain on the package versions
from v0.65.0 until the cumulative v0.70.0 checkpoint.

## Security Success Results

Certificate and SSH-key singleton responses share one typed result family:

```rust,ignore
use cloud_sdk_hetzner::serde::{HetznerSuccess, SecurityResource};

match success {
    HetznerSuccess::SecurityResource(SecurityResource::Certificate(cert)) => {
        let _ = (cert.id(), cert.name(), cert.status());
    }
    HetznerSuccess::SecurityResource(SecurityResource::SshKey(key)) => {
        key.try_with_public_key(|public_key| use_key(public_key))?;
    }
    _ => {}
}
```

Replace matches on `HetznerSuccess::Certificate` with the nested certificate
variant. List operations return `HetznerSuccess::SecurityResources`, and
certificate creation exposes its model through
`CompositeResult::security_resource`.

## Certificate Accessors

Certificate and nested status/use fields are private. Replace direct field
access with methods such as `id()`, `name()`, `certificate()`, `created()`,
`domain_names()`, `status()`, `used_by()`, `issuance()`, `renewal()`, and
`error()`. Certificate timestamps are `UtcTimestamp`; use `as_str()` when text
is required.

Secret-bearing security aggregates no longer support ordinary equality. Read
certificate chains and SSH public keys only through protected access paths,
and clear any caller-owned copies after use.

Use `SshKey::sha256_fingerprint()` for key identity comparisons. The existing
`SshKey::fingerprint()` method is retained as Hetzner's verified legacy MD5
text and should not be used as a collision-resistant identifier.

`CertificateError::code()` continues to provide the common classification.
Use `CertificateError::code_text()` when certificate-specific machine-readable
codes such as `issuance_failed` must drive diagnostics or remediation.
