# Migrating To v0.63.0

v0.63.0 is an internal source milestone after signed v0.62.0. No crate is
published; applications remain on the versions from the v0.60.0 checkpoint.

Checked ordinary Cloud responses now use dedicated result variants. Replace:

```rust,ignore
let HetznerSuccess::Resource(server) = decoded.success() else { return };
```

with:

```rust,ignore
let HetznerSuccess::CloudResource(server) = decoded.success() else { return };
assert_eq!(server.kind(), CloudResourceKind::Server);
```

Lists use `HetznerSuccess::CloudResources`, and Cloud create composites expose
the result through `CompositeResult::cloud_resource()`. The generic
`resource()` accessor remains for non-Cloud response families.

Use `fields()` when an operation needs a complete source or future field:

```rust,ignore
let status = server.fields().text("status");
let labels = server.fields().get("labels").and_then(CloudValue::as_object);
```

Single `get_location` responses now return `HetznerSuccess::Location`; paginated
location lists continue to return `HetznerSuccess::Locations`.

`Pricing` no longer implements `Eq` because its complete source tree can
contain finite fractional values. It remains `PartialEq` and its existing
summary accessors remain available.

`CloudValue`, `CloudObject`, every dedicated ordinary Cloud resource, and
`Pricing` deliberately do not implement infallible `Clone`. Replace `.clone()`
with `.try_clone()?` so allocation failure remains a recoverable
`ResponseModelError::Allocation`. Their `Debug` implementations expose only
shape or resource kind and redact identifiers and field values.

The next cumulative crates.io checkpoint is v0.65.0.
