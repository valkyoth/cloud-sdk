# Migrating To v0.67.0

v0.67.0 is an internal source milestone after signed v0.66.0. No crate is
published; applications remain on the package versions from v0.65.0 until the
cumulative v0.70.0 checkpoint.

## Storage Box Results

Match the dedicated result variants instead of generic resources:

```rust,ignore
use cloud_sdk_hetzner::serde::{HetznerSuccess, StorageBoxResource};

match success {
    HetznerSuccess::StorageBox(storage_box) => {
        let _ = (storage_box.id(), storage_box.name(), storage_box.status());
    }
    HetznerSuccess::StorageBoxSnapshot(snapshot) => {
        let _ = (snapshot.id(), snapshot.storage_box(), snapshot.created());
    }
    HetznerSuccess::Composite(composite) => {
        if let Some(StorageBoxResource::SubaccountReference(reference)) =
            composite.storage_box_resource()
        {
            let _ = (reference.id(), reference.storage_box());
        }
    }
    _ => {}
}
```

List operations use `StorageBoxes`, `StorageBoxTypes`,
`StorageBoxSnapshots`, and `StorageBoxSubaccounts`. Box and type pages expose
validated pagination metadata.

## Private Fields

Replace direct fields on `Money`, `Price`, `Deprecation`, `StorageBoxType`,
`StorageBox`, and `StorageBoxPage` with their same-named accessor methods.
The `created()` and deprecation timestamp accessors return canonical UTC text
directly.

These cleanup-owning aggregates no longer implement ordinary `Clone` or
field-based equality. Borrow values for inspection and avoid making
unprotected owned copies of usernames, hosts, descriptions, labels, or other
provider-returned account metadata.

## Typed Response Identity

Keep `Prepared<O>` and `AssociatedCheckedResponse<O>` typed through
`decode_associated_response` or `decode_associated_checked_response`. These
paths now reject Storage Box response identities that differ from the endpoint
used during preparation, including parent identities on snapshot/subaccount
lists and create references.

`Prepared::into_untyped` and `AssociatedCheckedResponse::into_untyped` remain
available for custom integrations, but explicitly discard this identity
binding. Custom decoders using those escape hatches must enforce equivalent
resource checks themselves.
