# Migrating To v0.62.0

v0.62.0 is an internal source milestone after signed v0.61.0. No crate is
published; applications remain on the versions from the v0.60.0 checkpoint.

The checked Hetzner success enum gains source-complete variants for locations,
certificates, and Storage Boxes. Code exhaustively matching
`HetznerSuccess` must add these variants. The enum was already a development
API and this change intentionally lands at the neutral freeze before 1.0.

`get_zone_zonefile` and certificate operations now retain their exact DNS and
security service identities during checked response decoding. Applications
that constructed synthetic prepared requests with the compute service for
these operations must use `DnsService` or `SecurityService` respectively.

The next cumulative crates.io checkpoint is v0.65.0.
