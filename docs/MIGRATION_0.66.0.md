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
