# Migrating To v0.86.0

v0.86 is additive and begins the v0.86-v0.90 internal milestone train.
Existing v0.85 published APIs are not removed. No crate is published for this
milestone.

## Source Versions

Repository users advance the facade source identity while independently
versioned provider crates remain at their latest published versions:

```toml
[dependencies]
cloud-sdk = "0.86.0"
cloud-sdk-hetzner = { version = "0.44.0", features = ["serde"] }
```

The next planned crates.io checkpoint is v0.90.0.

## Preparing A Reverse-DNS Read

```rust
use cloud_sdk::operation::PreparationStorage;
use cloud_sdk_hetzner::robot::{RobotIpAddress, RobotRdnsGetRequest};

let request = RobotRdnsGetRequest::new(RobotIpAddress::new("192.0.2.10")?);
let mut target = [0_u8; 64];
let mut body = [0_u8; 1];
let prepared = request.prepare_bound(PreparationStorage::new(&mut target, &mut body))?;
assert_eq!(
    prepared.as_untyped().transport_request().target().as_str(),
    "/rdns/192.0.2.10"
);
# Ok::<(), Box<dyn core::error::Error>>(())
```

List requests optionally accept a canonical IPv4 main-server address as their
`RobotIpAddress` filter. Record paths and responses may contain canonical IPv4
or IPv6 identities. Unfiltered lists use `decode_response`. A filtered Robot
response does not echo the server association, so obtain a freshly checked
`RobotIpList` and call `decode_response_with_inventory`; decoding rejects every
returned address that is not assigned to the exact filtered server. Provider
state can still change between those reads. Decode only through the
operation-associated prepared/checked wrapper so status, media type, bounds,
and request identity remain enforced; raw decoders are intentionally internal.

## Mutation Requirements

Construct PTR values with `RobotRdnsName`. Set and update require their exact
request-bound mutation permit. Delete requires the exact destructive permit.
All three operations deny automatic retry. After uncertain delivery, read the
current reverse-DNS state before deciding whether another mutation is safe.
