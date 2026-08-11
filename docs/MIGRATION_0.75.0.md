# Migrating To v0.75.0

v0.75.0 is the cumulative public checkpoint after v0.70.0.

```toml
[dependencies]
cloud-sdk = "0.75.0"
cloud-sdk-hetzner = "0.42.0"
cloud-sdk-reqwest = { version = "0.35.0", features = ["blocking-rustls"] }
cloud-sdk-testkit = "0.30.2"
```

Only add the crates and optional features used by the application.

## Robot Form Bodies

Construct ordered fields and keep the encoded guard alive through transport:

```rust
use cloud_sdk_hetzner::robot::{RobotForm, RobotFormField};

let fields = [
    RobotFormField::public("server[]", "192.0.2.10")?,
    RobotFormField::public("server[]", "192.0.2.11")?,
];
let form = RobotForm::new(&fields)?;
let mut storage = [0_u8; 128];
{
    let body = form.encode(&mut storage)?;
    assert!(body.as_bytes().starts_with(b"server%5B%5D="));
}
assert!(storage.iter().all(|byte| *byte == 0));
# Ok::<(), Box<dyn core::error::Error>>(())
```

Use `RobotFormField::sensitive` for password or other secret-bearing values.
This marks the body sensitive and guarantees destination cleanup, but it does
not clear the caller-owned source string or downstream transport copies.

## Existing Provider Clients

Named DNS, Security, and Console Storage Box clients accumulated after v0.70
are now included in the published provider package. Their operation signatures
and permit behavior are documented in the v0.71-v0.73 migration guides.

## Reqwest FIPS Removal

The experimental FIPS feature is unavailable in `cloud-sdk-reqwest 0.35.0`.
Use an ordinary optional rustls adapter or a caller-provided transport. Future
FIPS work is deferred until Brynja is ready and separately qualified.

## Not Yet Available

The Robot form codec is not a Robot client. Credentials, lockout-safe attempt
state, typed errors, endpoint operations, response models, and live execution
arrive in later milestones. Do not send credentials to an endpoint assembled
from tenant-controlled input.
