# Migrating Source Users To v0.71.0

v0.71.0 is an internal source milestone. The latest crates.io checkpoint
remains v0.70.0, and cumulative publication is deferred to v0.75.0.

## Adopting Named DNS Reads

Construct an official `HetznerClient::dns` from an endpoint-bound authenticated
transport. Build the same `AssociatedOperation<O, ...>` used by existing
generic code, acquire one `ClientWorkspaceLease`, and call the operation's
named blocking, `Send` async, or local-async method.

The compile-checked
[`dns_client` example](../crates/cloud-sdk-hetzner/examples/dns_client.rs)
shows a paginated zone list against the deterministic testkit transport.

## Adopting Named DNS State Changes

For mutation and destructive operations:

1. Call the named `prepare_<operation>` method with a
   `PreparationStorageGuard`.
2. Review the complete method, target, query, and body.
3. Build and fingerprint an `AssociatedPlanConfirmation`.
4. Create the matching mutation or destructive permit.
5. Begin one attempt and pass it to the named executor method.

There is no direct state-changing method and no implicit retry. Zonefile and
TSIG data remains caller-owned and must be cleared after transport use; the
SDK clears its complete guarded and workspace buffers.

## FIPS Feature Removal

Remove `blocking-rustls-fips`, `FipsTlsPolicy`, and FIPS builder calls from
source consumers. Use `blocking-rustls`, `blocking-rustls-webpki-roots`, or
`async-rustls` when ordinary rustls transport is appropriate.

FIPS support is not replaced in v0.71 and is not part of the cloud-sdk 1.0
scope. It is deferred until Brynja satisfies
[`FIPS_DEFERMENT.md`](FIPS_DEFERMENT.md). Applications requiring validated
cryptography must qualify their own transport in the meantime.
