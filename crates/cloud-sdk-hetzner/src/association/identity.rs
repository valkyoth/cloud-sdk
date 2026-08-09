//! Opaque response identities captured from typed endpoint components.

/// Runtime identity expected from a source-locked response model.
///
/// This value is intentionally crate-private and has no `Debug`
/// implementation so request identifiers cannot enter diagnostics through the
/// association layer.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
#[cfg_attr(not(feature = "serde"), allow(dead_code))]
pub(crate) enum ExpectedResponseIdentity {
    /// The operation has no currently modeled response-identity contract.
    #[default]
    None,
    /// One Storage Box must match this identifier.
    StorageBox(u64),
    /// One Storage Box type must match this identifier.
    StorageBoxType(u64),
    /// Snapshot responses must match this parent and optional snapshot.
    StorageBoxSnapshot {
        storage_box: u64,
        snapshot: Option<u64>,
    },
    /// Subaccount responses must match this parent and optional subaccount.
    StorageBoxSubaccount {
        storage_box: u64,
        subaccount: Option<u64>,
    },
}
