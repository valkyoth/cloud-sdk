use crate::prepared::EndpointWire;
use crate::storage::storage_boxes::{
    StorageBoxEndpoint, StorageBoxId, StorageBoxSnapshotEndpoint, StorageBoxSnapshotId,
    StorageBoxSubaccountEndpoint, StorageBoxSubaccountId, StorageBoxTypeEndpoint, StorageBoxTypeId,
};

use super::ExpectedResponseIdentity;

#[test]
fn every_storage_response_identity_policy_is_source_locked() {
    let Some(storage_box) = StorageBoxId::new(17) else {
        unreachable!("Storage Box identity fixture failed")
    };
    let Some(snapshot) = StorageBoxSnapshotId::new(23) else {
        unreachable!("snapshot identity fixture failed")
    };
    let Some(subaccount) = StorageBoxSubaccountId::new(29) else {
        unreachable!("subaccount identity fixture failed")
    };
    let Some(storage_box_type) = StorageBoxTypeId::new(31) else {
        unreachable!("Storage Box type identity fixture failed")
    };

    for endpoint in [StorageBoxEndpoint::List, StorageBoxEndpoint::Create] {
        assert!(endpoint.expected_response_identity() == ExpectedResponseIdentity::None);
    }
    for endpoint in [
        StorageBoxEndpoint::Get(storage_box),
        StorageBoxEndpoint::Update(storage_box),
    ] {
        assert!(
            endpoint.expected_response_identity()
                == ExpectedResponseIdentity::StorageBox(storage_box.get())
        );
    }
    for endpoint in [
        StorageBoxEndpoint::Delete(storage_box),
        StorageBoxEndpoint::ListFolders(storage_box),
    ] {
        assert!(endpoint.expected_response_identity() == ExpectedResponseIdentity::None);
    }

    assert!(
        StorageBoxTypeEndpoint::List.expected_response_identity() == ExpectedResponseIdentity::None
    );
    assert!(
        StorageBoxTypeEndpoint::Get(storage_box_type).expected_response_identity()
            == ExpectedResponseIdentity::StorageBoxType(storage_box_type.get())
    );

    for endpoint in [
        StorageBoxSnapshotEndpoint::List(storage_box),
        StorageBoxSnapshotEndpoint::Create(storage_box),
    ] {
        assert!(
            endpoint.expected_response_identity()
                == ExpectedResponseIdentity::StorageBoxSnapshot {
                    storage_box: storage_box.get(),
                    snapshot: None,
                }
        );
    }
    for endpoint in [
        StorageBoxSnapshotEndpoint::Get(storage_box, snapshot),
        StorageBoxSnapshotEndpoint::Update(storage_box, snapshot),
    ] {
        assert!(
            endpoint.expected_response_identity()
                == ExpectedResponseIdentity::StorageBoxSnapshot {
                    storage_box: storage_box.get(),
                    snapshot: Some(snapshot.get()),
                }
        );
    }
    assert!(
        StorageBoxSnapshotEndpoint::Delete(storage_box, snapshot).expected_response_identity()
            == ExpectedResponseIdentity::None
    );

    for endpoint in [
        StorageBoxSubaccountEndpoint::List(storage_box),
        StorageBoxSubaccountEndpoint::Create(storage_box),
    ] {
        assert!(
            endpoint.expected_response_identity()
                == ExpectedResponseIdentity::StorageBoxSubaccount {
                    storage_box: storage_box.get(),
                    subaccount: None,
                }
        );
    }
    for endpoint in [
        StorageBoxSubaccountEndpoint::Get(storage_box, subaccount),
        StorageBoxSubaccountEndpoint::Update(storage_box, subaccount),
    ] {
        assert!(
            endpoint.expected_response_identity()
                == ExpectedResponseIdentity::StorageBoxSubaccount {
                    storage_box: storage_box.get(),
                    subaccount: Some(subaccount.get()),
                }
        );
    }
    assert!(
        StorageBoxSubaccountEndpoint::Delete(storage_box, subaccount).expected_response_identity()
            == ExpectedResponseIdentity::None
    );
}
