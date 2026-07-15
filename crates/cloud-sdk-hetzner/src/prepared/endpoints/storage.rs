//! Console Storage Box endpoint adapters.

use cloud_sdk::operation::{CostIntent, PreparationStorage, PrepareOperation, PreparedRequest};

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
        StorageBoxEndpoint::Delete(_) => true,
        _ => false,
    },
    match endpoint {
        StorageBoxEndpoint::Create => CostIntent::MayIncurCost,
        _ => CostIntent::NoKnownCost,
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
    false,
    CostIntent::NoKnownCost
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
        StorageBoxActionEndpoint::ChangeProtection(_)
            | StorageBoxActionEndpoint::DisableSnapshotPlan(_)
            | StorageBoxActionEndpoint::ResetPassword(_)
            | StorageBoxActionEndpoint::RollbackSnapshot(_) => true,
        _ => false,
    },
    match endpoint {
        StorageBoxActionEndpoint::ChangeType(_) => CostIntent::MayIncurCost,
        _ => CostIntent::NoKnownCost,
    }
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
        StorageBoxSnapshotEndpoint::Delete(_, _) => true,
        _ => false,
    },
    CostIntent::NoKnownCost
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
        StorageBoxSubaccountEndpoint::Delete(_, _) => true,
        _ => false,
    },
    CostIntent::NoKnownCost
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
        StorageBoxSubaccountActionEndpoint::ResetPassword(_, _) => true,
        _ => false,
    },
    CostIntent::NoKnownCost
);
