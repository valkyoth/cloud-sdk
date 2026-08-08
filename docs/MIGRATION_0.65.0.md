# Migrating To v0.65.0

v0.65.0 is the public checkpoint after v0.60.0. Update independently versioned
dependencies that your application uses:

```toml
[dependencies]
cloud-sdk = "0.65.0"
cloud-sdk-hetzner = { version = "0.40.0", features = ["serde"] }
cloud-sdk-reqwest = { version = "0.34.0", features = ["blocking-rustls"] }

[dev-dependencies]
cloud-sdk-testkit = "0.30.0"
```

`cloud-sdk-sanitization` remains at `0.18.0` and is not republished.

## DNS Success Variants

Code matching generic DNS resources must select the dedicated variants:

```rust
use cloud_sdk_hetzner::serde::{DnsResource, HetznerSuccess};

match success {
    HetznerSuccess::DnsResource(DnsResource::Zone(zone)) => {
        let _ = (zone.id(), zone.name(), zone.ttl());
    }
    HetznerSuccess::DnsResource(DnsResource::Rrset(rrset)) => {
        let _ = (rrset.id(), rrset.record_type().as_str(), rrset.records());
    }
    _ => {}
}
```

Paged DNS operations return `HetznerSuccess::DnsResources`. Zone/RRSet create
responses expose the object through `CompositeResult::dns_resource` rather than
`resource`.

## TSIG And Future RR Types

Read returned TSIG keys only through `PrimaryNameserver::try_with_tsig_key` and
avoid copying them into ordinary strings. A response `DnsRrsetType` may have
`known() == None`; inspect `as_str()` for diagnostics or storage, but do not
send it through a request constructor until a later source-lock update assigns
explicit semantics.
