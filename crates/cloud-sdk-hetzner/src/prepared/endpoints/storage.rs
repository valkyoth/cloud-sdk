//! Console Storage Box endpoint adapters.

use cloud_sdk::operation::{CostIntent, PreparationStorage, PrepareOperation, PreparedRequest};

use crate::prepared::operation::OperationClass;
use crate::storage::storage_boxes::{
    StorageBoxActionEndpoint, StorageBoxActionListRequest, StorageBoxEndpoint,
    StorageBoxListRequest, StorageBoxSnapshotEndpoint, StorageBoxSnapshotListRequest,
    StorageBoxSubaccountActionEndpoint, StorageBoxSubaccountEndpoint,
    StorageBoxSubaccountListRequest, StorageBoxTypeEndpoint, StorageBoxTypeListRequest,
};

use super::super::{
    HetznerPreparationError, HetznerPreparedOperation, RequestShape, ResponseProfile,
};

endpoint_wire!(
    StorageBoxEndpoint,
    endpoint => match endpoint {
        StorageBoxEndpoint::List => RequestShape::OptionalQuery,
        StorageBoxEndpoint::Create | StorageBoxEndpoint::Update(_) => RequestShape::RequiredJson,
        StorageBoxEndpoint::Get(_)
        | StorageBoxEndpoint::Delete(_)
        | StorageBoxEndpoint::ListFolders(_) => RequestShape::None,
    },
    match endpoint {
        StorageBoxEndpoint::Create | StorageBoxEndpoint::Delete(_) => ResponseProfile::JsonCreated,
        _ => ResponseProfile::JsonOk,
    },
    match endpoint {
        StorageBoxEndpoint::List => "list_storage_boxes",
        StorageBoxEndpoint::Create => "create_storage_box",
        StorageBoxEndpoint::Get(_) => "get_storage_box",
        StorageBoxEndpoint::Update(_) => "update_storage_box",
        StorageBoxEndpoint::Delete(_) => "delete_storage_box",
        StorageBoxEndpoint::ListFolders(_) => "list_storage_box_folders",
    },
    match endpoint {
        StorageBoxEndpoint::List
        | StorageBoxEndpoint::Get(_)
        | StorageBoxEndpoint::ListFolders(_) => OperationClass::ReadOnly,
        StorageBoxEndpoint::Create => OperationClass::NonIdempotentMutation,
        StorageBoxEndpoint::Update(_) => OperationClass::IdempotentMutation,
        StorageBoxEndpoint::Delete(_) => OperationClass::IdempotentDestructive,
    },
    match endpoint {
        StorageBoxEndpoint::Create => CostIntent::MayIncurCost,
        _ => CostIntent::NoKnownCost,
    },
    identity endpoint => match endpoint {
        StorageBoxEndpoint::Get(id) | StorageBoxEndpoint::Update(id) => {
            crate::association::ExpectedResponseIdentity::StorageBox(id.get())
        }
        _ => crate::association::ExpectedResponseIdentity::None,
    }
);

query_wire!(StorageBoxListRequest<'_>, request => {
    let _ = request;
    StorageBoxEndpoint::List
});

endpoint_wire!(
    StorageBoxTypeEndpoint,
    endpoint => match endpoint {
        StorageBoxTypeEndpoint::List => RequestShape::OptionalQuery,
        StorageBoxTypeEndpoint::Get(_) => RequestShape::None,
    },
    ResponseProfile::JsonOk,
    match endpoint {
        StorageBoxTypeEndpoint::List => "list_storage_box_types",
        StorageBoxTypeEndpoint::Get(_) => "get_storage_box_type",
    },
    OperationClass::ReadOnly,
    CostIntent::NoKnownCost,
    identity endpoint => match endpoint {
        StorageBoxTypeEndpoint::Get(id) => {
            crate::association::ExpectedResponseIdentity::StorageBoxType(id.get())
        }
        StorageBoxTypeEndpoint::List => crate::association::ExpectedResponseIdentity::None,
    }
);

query_wire!(StorageBoxTypeListRequest<'_>, request => {
    let _ = request;
    StorageBoxTypeEndpoint::List
});

endpoint_wire!(
    StorageBoxActionEndpoint,
    endpoint => match endpoint {
        StorageBoxActionEndpoint::ListAll
        | StorageBoxActionEndpoint::ListForStorageBox(_) => RequestShape::OptionalQuery,
        StorageBoxActionEndpoint::Get(_) | StorageBoxActionEndpoint::DisableSnapshotPlan(_) => {
            RequestShape::None
        }
        _ => RequestShape::RequiredJson,
    },
    match endpoint {
        StorageBoxActionEndpoint::ListAll
        | StorageBoxActionEndpoint::Get(_)
        | StorageBoxActionEndpoint::ListForStorageBox(_) => ResponseProfile::JsonOk,
        _ => ResponseProfile::JsonCreated,
    },
    match endpoint {
        StorageBoxActionEndpoint::ListAll => "list_storage_boxes_actions",
        StorageBoxActionEndpoint::Get(_) => "get_storage_boxes_action",
        StorageBoxActionEndpoint::ListForStorageBox(_) => "list_storage_box_actions",
        StorageBoxActionEndpoint::ChangeProtection(_) => "change_storage_box_protection",
        StorageBoxActionEndpoint::ChangeType(_) => "change_storage_box_type",
        StorageBoxActionEndpoint::DisableSnapshotPlan(_) => "disable_storage_box_snapshot_plan",
        StorageBoxActionEndpoint::EnableSnapshotPlan(_) => "enable_storage_box_snapshot_plan",
        StorageBoxActionEndpoint::ResetPassword(_) => "reset_storage_box_password",
        StorageBoxActionEndpoint::RollbackSnapshot(_) => "rollback_storage_box_snapshot",
        StorageBoxActionEndpoint::UpdateAccessSettings(_) => "update_storage_box_access_settings",
    },
    match endpoint {
        StorageBoxActionEndpoint::ListAll
        | StorageBoxActionEndpoint::Get(_)
        | StorageBoxActionEndpoint::ListForStorageBox(_) => OperationClass::ReadOnly,
        StorageBoxActionEndpoint::ChangeProtection(_)
            | StorageBoxActionEndpoint::DisableSnapshotPlan(_)
            | StorageBoxActionEndpoint::ResetPassword(_)
            | StorageBoxActionEndpoint::RollbackSnapshot(_) => {
                OperationClass::NonIdempotentDestructive
            }
        StorageBoxActionEndpoint::ChangeType(_)
        | StorageBoxActionEndpoint::EnableSnapshotPlan(_)
        | StorageBoxActionEndpoint::UpdateAccessSettings(_) => {
            OperationClass::NonIdempotentMutation
        }
    },
    match endpoint {
        StorageBoxActionEndpoint::ChangeType(_) => CostIntent::MayIncurCost,
        _ => CostIntent::NoKnownCost,
    },
    identity endpoint => { let _ = endpoint; crate::association::ExpectedResponseIdentity::None }
);

impl crate::prepared::QueryWire for StorageBoxActionListRequest {
    fn write_query(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        self.write_query(output)
            .map_err(|_| HetznerPreparationError::Query)
    }

    fn operation_key(self) -> &'static str {
        "list_storage_boxes_actions"
    }

    fn accepts_operation(self, operation_key: &str) -> bool {
        match operation_key {
            "list_storage_boxes_actions" | "list_storage_box_actions" => true,
            _ => false,
        }
    }
}

impl PrepareOperation for StorageBoxActionListRequest {
    type Error = HetznerPreparationError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        HetznerPreparedOperation::query(StorageBoxActionEndpoint::ListAll, *self).prepare(storage)
    }
}

endpoint_wire!(
    StorageBoxSnapshotEndpoint,
    endpoint => match endpoint {
        StorageBoxSnapshotEndpoint::List(_) => RequestShape::OptionalQuery,
        StorageBoxSnapshotEndpoint::Create(_) | StorageBoxSnapshotEndpoint::Update(_, _) => {
            RequestShape::RequiredJson
        }
        StorageBoxSnapshotEndpoint::Get(_, _) | StorageBoxSnapshotEndpoint::Delete(_, _) => {
            RequestShape::None
        }
    },
    match endpoint {
        StorageBoxSnapshotEndpoint::Create(_) | StorageBoxSnapshotEndpoint::Delete(_, _) => {
            ResponseProfile::JsonCreated
        }
        _ => ResponseProfile::JsonOk,
    },
    match endpoint {
        StorageBoxSnapshotEndpoint::List(_) => "list_storage_box_snapshots",
        StorageBoxSnapshotEndpoint::Create(_) => "create_storage_box_snapshot",
        StorageBoxSnapshotEndpoint::Get(_, _) => "get_storage_box_snapshot",
        StorageBoxSnapshotEndpoint::Update(_, _) => "update_storage_box_snapshot",
        StorageBoxSnapshotEndpoint::Delete(_, _) => "delete_storage_box_snapshot",
    },
    match endpoint {
        StorageBoxSnapshotEndpoint::List(_) | StorageBoxSnapshotEndpoint::Get(_, _) => {
            OperationClass::ReadOnly
        }
        StorageBoxSnapshotEndpoint::Create(_) => OperationClass::NonIdempotentMutation,
        StorageBoxSnapshotEndpoint::Update(_, _) => OperationClass::IdempotentMutation,
        StorageBoxSnapshotEndpoint::Delete(_, _) => OperationClass::IdempotentDestructive,
    },
    CostIntent::NoKnownCost,
    identity endpoint => match endpoint {
        StorageBoxSnapshotEndpoint::List(storage_box)
        | StorageBoxSnapshotEndpoint::Create(storage_box) => {
            crate::association::ExpectedResponseIdentity::StorageBoxSnapshot {
                storage_box: storage_box.get(),
                snapshot: None,
            }
        }
        StorageBoxSnapshotEndpoint::Get(storage_box, snapshot)
        | StorageBoxSnapshotEndpoint::Update(storage_box, snapshot) => {
            crate::association::ExpectedResponseIdentity::StorageBoxSnapshot {
                storage_box: storage_box.get(),
                snapshot: Some(snapshot.get()),
            }
        }
        StorageBoxSnapshotEndpoint::Delete(_, _) => {
            crate::association::ExpectedResponseIdentity::None
        }
    }
);

impl crate::prepared::QueryWire for StorageBoxSnapshotListRequest<'_> {
    fn write_query(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        self.write_query(output)
            .map_err(|_| HetznerPreparationError::Query)
    }

    fn operation_key(self) -> &'static str {
        "list_storage_box_snapshots"
    }
}

endpoint_wire!(
    StorageBoxSubaccountEndpoint,
    endpoint => match endpoint {
        StorageBoxSubaccountEndpoint::List(_) => RequestShape::OptionalQuery,
        StorageBoxSubaccountEndpoint::Create(_) | StorageBoxSubaccountEndpoint::Update(_, _) => {
            RequestShape::RequiredJson
        }
        StorageBoxSubaccountEndpoint::Get(_, _)
        | StorageBoxSubaccountEndpoint::Delete(_, _) => RequestShape::None,
    },
    match endpoint {
        StorageBoxSubaccountEndpoint::Create(_) | StorageBoxSubaccountEndpoint::Delete(_, _) => {
            ResponseProfile::JsonCreated
        }
        _ => ResponseProfile::JsonOk,
    },
    match endpoint {
        StorageBoxSubaccountEndpoint::List(_) => "list_storage_box_subaccounts",
        StorageBoxSubaccountEndpoint::Create(_) => "create_storage_box_subaccount",
        StorageBoxSubaccountEndpoint::Get(_, _) => "get_storage_box_subaccount",
        StorageBoxSubaccountEndpoint::Update(_, _) => "update_storage_box_subaccount",
        StorageBoxSubaccountEndpoint::Delete(_, _) => "delete_storage_box_subaccount",
    },
    match endpoint {
        StorageBoxSubaccountEndpoint::List(_) | StorageBoxSubaccountEndpoint::Get(_, _) => {
            OperationClass::ReadOnly
        }
        StorageBoxSubaccountEndpoint::Create(_) => OperationClass::NonIdempotentMutation,
        StorageBoxSubaccountEndpoint::Update(_, _) => OperationClass::IdempotentMutation,
        StorageBoxSubaccountEndpoint::Delete(_, _) => OperationClass::IdempotentDestructive,
    },
    CostIntent::NoKnownCost,
    identity endpoint => match endpoint {
        StorageBoxSubaccountEndpoint::List(storage_box)
        | StorageBoxSubaccountEndpoint::Create(storage_box) => {
            crate::association::ExpectedResponseIdentity::StorageBoxSubaccount {
                storage_box: storage_box.get(),
                subaccount: None,
            }
        }
        StorageBoxSubaccountEndpoint::Get(storage_box, subaccount)
        | StorageBoxSubaccountEndpoint::Update(storage_box, subaccount) => {
            crate::association::ExpectedResponseIdentity::StorageBoxSubaccount {
                storage_box: storage_box.get(),
                subaccount: Some(subaccount.get()),
            }
        }
        StorageBoxSubaccountEndpoint::Delete(_, _) => {
            crate::association::ExpectedResponseIdentity::None
        }
    }
);

impl crate::prepared::QueryWire for StorageBoxSubaccountListRequest<'_> {
    fn write_query(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        self.write_query(output)
            .map_err(|_| HetznerPreparationError::Query)
    }

    fn operation_key(self) -> &'static str {
        "list_storage_box_subaccounts"
    }
}

endpoint_wire!(
    StorageBoxSubaccountActionEndpoint,
    endpoint => RequestShape::RequiredJson,
    ResponseProfile::JsonCreated,
    match endpoint {
        StorageBoxSubaccountActionEndpoint::ChangeHomeDirectory(_, _) => "change_storage_box_subaccount_home_directory",
        StorageBoxSubaccountActionEndpoint::ResetPassword(_, _) => "reset_storage_box_subaccount_password",
        StorageBoxSubaccountActionEndpoint::UpdateAccessSettings(_, _) => "update_storage_box_subaccount_access_settings",
    },
    match endpoint {
        StorageBoxSubaccountActionEndpoint::ResetPassword(_, _) => {
            OperationClass::NonIdempotentDestructive
        }
        StorageBoxSubaccountActionEndpoint::ChangeHomeDirectory(_, _)
        | StorageBoxSubaccountActionEndpoint::UpdateAccessSettings(_, _) => {
            OperationClass::NonIdempotentMutation
        }
    },
    CostIntent::NoKnownCost,
    identity endpoint => { let _ = endpoint; crate::association::ExpectedResponseIdentity::None }
);
