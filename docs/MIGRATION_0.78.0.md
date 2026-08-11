# Migrating Source Users To v0.78.0

v0.78 is additive. Existing v0.77 Robot credential, form, and error code does
not require changes.

## Preparing A Server Read

```rust
use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk_hetzner::robot::{RobotServerGetRequest, RobotServerNumber};

let number = RobotServerNumber::new(321).ok_or("server number must be positive")?;
let request = RobotServerGetRequest::new(number);
let mut target = [0_u8; 64];
let mut body = [0_u8; 1];
let prepared = request.prepare(PreparationStorage::new(&mut target, &mut body))?;
assert_eq!(prepared.transport_request().target().as_str(), "/server/321");
# Ok::<(), Box<dyn core::error::Error>>(())
```

Use `RobotServerListRequest::new()` for the collection and
`RobotServerUpdateRequest::rename(number, name)` for the sole current update.
There is intentionally no constructor that accepts an IPv4 address as the
server path identity.

With `serde`, validate the transport response through the prepared request,
then pass the resulting guard to `request.decode_response(checked)`. The get
and update methods verify the response server number before returning.

## Model Differences

- `RobotServerSummary::subnets()` returns `None` for JSON `null` and
  `Some(&[])` for an empty array.
- Provider text uses closure-scoped accessors such as `try_with_name`.
- Unknown statuses, extra fields, duplicate identities, invalid dates,
  malformed addresses, and subnets with host bits set are rejected.
- A missing or zero `linked_storagebox` maps to `None` due to the documented
  official-source inconsistency.

The crate package remains 0.42.0 until cumulative publication at v0.80.0.
