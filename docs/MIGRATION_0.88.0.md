# Migrating Source Users To v0.88

v0.88 is an internal cumulative milestone. No crate is published until the
v0.90 checkpoint.

Source users should update the core dependency to `0.88.0`; the Hetzner package
version remains `0.44.0` during the cumulative train.

```toml
[dependencies]
cloud-sdk = "0.88.0"
cloud-sdk-hetzner = { version = "0.44.0", features = ["serde"] }
```

Construct a bounded key create request and retain the typed association:

```rust
use cloud_sdk::operation::PreparationStorage;
use cloud_sdk_hetzner::robot::{
    RobotSshKeyCreateRequest, RobotSshKeyData, RobotSshKeyName,
};

let name = RobotSshKeyName::new("deploy-key")?;
let data = RobotSshKeyData::new(
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIMockKey",
)?;
let request = RobotSshKeyCreateRequest::new(name, data);
let mut path = [0_u8; 64];
let mut body = [0_u8; 16_384];
let prepared = request.prepare_bound(PreparationStorage::new(&mut path, &mut body))?;
# let _ = prepared;
# Ok::<(), Box<dyn core::error::Error>>(())
```

Create and rename requests require the existing strong-digest mutation permit
flow; delete requires destructive authority. After checked decoding, compare
`RobotSshKey::sha256_fingerprint` for strong identity. Use the protected MD5
fingerprint only where Robot requires it in an endpoint path.
