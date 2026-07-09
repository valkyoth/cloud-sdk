use super::{
    ENDPOINT_GROUPS, RESOURCE_LOCAL_ACTION_GET_DEFERRED, SnapshotPlanDayOfMonth,
    SnapshotPlanDayOfWeek, SnapshotPlanHour, SnapshotPlanMaxSnapshots, SnapshotPlanMinute,
    StorageBoxActionEndpoint, StorageBoxActionListRequest, StorageBoxActionSortField,
    StorageBoxChangeHomeDirectoryRequest, StorageBoxCreateRequest, StorageBoxEndpoint,
    StorageBoxHomeDirectory, StorageBoxId, StorageBoxListRequest, StorageBoxLocation,
    StorageBoxName, StorageBoxPassword, StorageBoxProtectionRequest, StorageBoxRequestError,
    StorageBoxResetPasswordRequest, StorageBoxRollbackSnapshotRequest, StorageBoxSnapshotEndpoint,
    StorageBoxSnapshotId, StorageBoxSnapshotListRequest, StorageBoxSnapshotPlanRequest,
    StorageBoxSnapshotRef, StorageBoxSnapshotSortField, StorageBoxSortField,
    StorageBoxSubaccountActionEndpoint, StorageBoxSubaccountCreateRequest,
    StorageBoxSubaccountEndpoint, StorageBoxSubaccountId, StorageBoxSubaccountListRequest,
    StorageBoxSubaccountSortField, StorageBoxSubaccountUsername, StorageBoxTypeEndpoint,
    StorageBoxTypeId, StorageBoxTypeListRequest, StorageBoxTypeRef,
};
use crate::EndpointGroup;
use crate::actions::{ActionId, ActionStatus};
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::ApiBaseUrl;

#[test]
fn storage_box_groups_are_owned_by_storage_module() {
    assert!(ENDPOINT_GROUPS.contains(&EndpointGroup::StorageBoxes));
    assert!(ENDPOINT_GROUPS.contains(&EndpointGroup::StorageBoxActions));
    assert!(ENDPOINT_GROUPS.contains(&EndpointGroup::StorageBoxSubaccounts));
    let resource_local_action_get_deferred = RESOURCE_LOCAL_ACTION_GET_DEFERRED;
    assert!(resource_local_action_get_deferred);
}

#[test]
fn storage_box_paths_match_api_matrix() {
    let id = storage_box_id();
    assert!(id.is_some(), "fixture storage box ID must validate");
    let Some(id) = id else { return };
    let type_id = StorageBoxTypeId::new(3);
    assert!(type_id.is_some(), "fixture type ID must validate");
    let Some(type_id) = type_id else { return };
    let action_id = ActionId::new(9);
    assert!(action_id.is_some(), "fixture action ID must validate");
    let Some(action_id) = action_id else { return };
    let snapshot_id = StorageBoxSnapshotId::new(7);
    assert!(snapshot_id.is_some(), "fixture snapshot ID must validate");
    let Some(snapshot_id) = snapshot_id else {
        return;
    };
    let subaccount_id = StorageBoxSubaccountId::new(8);
    assert!(
        subaccount_id.is_some(),
        "fixture subaccount ID must validate"
    );
    let Some(subaccount_id) = subaccount_id else {
        return;
    };
    let mut output = [0u8; 128];

    assert_path(
        StorageBoxEndpoint::List.write_path(&mut output),
        &output,
        "/storage_boxes",
    );
    assert_path(
        StorageBoxEndpoint::Get(id).write_path(&mut output),
        &output,
        "/storage_boxes/42",
    );
    assert_path(
        StorageBoxEndpoint::ListFolders(id).write_path(&mut output),
        &output,
        "/storage_boxes/42/folders",
    );
    assert_eq!(
        StorageBoxEndpoint::Create.api_base_url(),
        ApiBaseUrl::HetznerV1
    );
    assert_eq!(StorageBoxEndpoint::Delete(id).method().as_str(), "DELETE");

    assert_path(
        StorageBoxTypeEndpoint::List.write_path(&mut output),
        &output,
        "/storage_box_types",
    );
    assert_path(
        StorageBoxTypeEndpoint::Get(type_id).write_path(&mut output),
        &output,
        "/storage_box_types/3",
    );

    assert_path(
        StorageBoxActionEndpoint::ListAll.write_path(&mut output),
        &output,
        "/storage_boxes/actions",
    );
    assert_path(
        StorageBoxActionEndpoint::Get(action_id).write_path(&mut output),
        &output,
        "/storage_boxes/actions/9",
    );
    assert_path(
        StorageBoxActionEndpoint::EnableSnapshotPlan(id).write_path(&mut output),
        &output,
        "/storage_boxes/42/actions/enable_snapshot_plan",
    );
    assert_eq!(
        StorageBoxActionEndpoint::ResetPassword(id).endpoint_group(),
        EndpointGroup::StorageBoxActions
    );

    assert_path(
        StorageBoxSnapshotEndpoint::Delete(id, snapshot_id).write_path(&mut output),
        &output,
        "/storage_boxes/42/snapshots/7",
    );
    assert_path(
        StorageBoxSubaccountEndpoint::Update(id, subaccount_id).write_path(&mut output),
        &output,
        "/storage_boxes/42/subaccounts/8",
    );
    assert_path(
        StorageBoxSubaccountActionEndpoint::ResetPassword(id, subaccount_id)
            .write_path(&mut output),
        &output,
        "/storage_boxes/42/subaccounts/8/actions/reset_subaccount_password",
    );
}

#[test]
fn storage_box_queries_match_source_locked_parameters() {
    let selector = LabelSelector::new("env=prod");
    assert!(selector.is_ok(), "fixture selector must validate");
    let Ok(selector) = selector else { return };
    let name = StorageBoxName::new("backup-box");
    assert!(name.is_ok(), "fixture name must validate");
    let Ok(name) = name else { return };
    let page = Page::new(2);
    assert!(page.is_ok(), "fixture page must validate");
    let Ok(page) = page else { return };
    let per_page = PerPage::new(25);
    assert!(per_page.is_ok(), "fixture per_page must validate");
    let Ok(per_page) = per_page else { return };
    let action_id = ActionId::new(9);
    assert!(action_id.is_some(), "fixture action ID must validate");
    let Some(action_id) = action_id else { return };
    let username = StorageBoxSubaccountUsername::new("u42");
    assert!(username.is_ok(), "fixture username validates");
    let Ok(username) = username else { return };
    let mut output = [0u8; 160];

    let query = StorageBoxListRequest::new()
        .with_label_selector(selector)
        .with_name(name)
        .with_page(page)
        .with_per_page(per_page)
        .with_sort(StorageBoxSortField::StatsSize, SortDirection::Desc)
        .write_query(&mut output);
    assert_query(
        query,
        &output,
        "label_selector=env%3Dprod&name=backup-box&page=2&per_page=25&sort=stats.size%3Adesc",
    );

    let query = StorageBoxTypeListRequest::new()
        .with_name(name)
        .with_page(page)
        .with_per_page(per_page)
        .write_query(&mut output);
    assert_query(query, &output, "name=backup-box&page=2&per_page=25");

    let query = StorageBoxActionListRequest::new()
        .with_action_id(action_id)
        .with_page(page)
        .with_per_page(per_page)
        .with_sort(StorageBoxActionSortField::Started, SortDirection::Asc)
        .with_status(ActionStatus::Running)
        .write_query(&mut output);
    assert_query(
        query,
        &output,
        "id=9&page=2&per_page=25&sort=started%3Aasc&status=running",
    );

    let query = StorageBoxSnapshotListRequest::new()
        .with_is_automatic(true)
        .with_label_selector(selector)
        .with_name(name)
        .with_sort(StorageBoxSnapshotSortField::Created, SortDirection::Desc)
        .write_query(&mut output);
    assert_query(
        query,
        &output,
        "is_automatic=true&label_selector=env%3Dprod&name=backup-box&sort=created%3Adesc",
    );

    let query = StorageBoxSubaccountListRequest::new()
        .with_label_selector(selector)
        .with_name(name)
        .with_sort(StorageBoxSubaccountSortField::Created, SortDirection::Desc)
        .with_username(username)
        .write_query(&mut output);
    assert_query(
        query,
        &output,
        "label_selector=env%3Dprod&name=backup-box&sort=created%3Adesc&username=u42",
    );
}

#[test]
fn storage_box_body_markers_validate_required_fields_and_secrets() {
    let name = StorageBoxName::new("backup-box");
    assert!(name.is_ok(), "fixture name must validate");
    let Ok(name) = name else { return };
    let location = StorageBoxLocation::new("fsn1");
    assert!(location.is_ok(), "fixture location must validate");
    let Ok(location) = location else { return };
    let box_type = StorageBoxTypeRef::new("bx20");
    assert!(box_type.is_ok(), "fixture type ref validates");
    let Ok(box_type) = box_type else { return };
    let password = StorageBoxPassword::new(r#"a"b\c"#);
    assert!(password.is_ok(), "fixture password validates");
    let Ok(password) = password else { return };
    let mut output = [0u8; 16];

    assert_eq!(
        StorageBoxCreateRequest::try_new(Some(name), Some(location), Some(box_type), None),
        Err(StorageBoxRequestError::MissingRequiredField)
    );
    assert_eq!(
        StorageBoxCreateRequest::try_new(
            Some(name),
            Some(location),
            Some(box_type),
            Some(password)
        )
        .map(StorageBoxCreateRequest::endpoint),
        Ok(StorageBoxEndpoint::Create)
    );
    assert_query(
        password.write_json_string(&mut output),
        &output,
        r#""a\"b\\c""#,
    );
    assert_eq!(
        StorageBoxResetPasswordRequest::try_new(None),
        Err(StorageBoxRequestError::MissingRequiredField)
    );
    assert!(StorageBoxProtectionRequest::new(true).delete());
}

#[test]
fn storage_box_snapshot_and_subaccount_markers_validate_boundaries() {
    let id = storage_box_id();
    assert!(id.is_some(), "fixture storage box ID must validate");
    let Some(id) = id else { return };
    let snapshot = StorageBoxSnapshotRef::new("snapshot-1");
    assert!(snapshot.is_ok(), "fixture snapshot validates");
    let Ok(snapshot) = snapshot else { return };
    assert_eq!(SnapshotPlanMaxSnapshots::new(0), None);
    assert_eq!(SnapshotPlanMinute::new(60), None);
    assert_eq!(SnapshotPlanHour::new(24), None);
    assert_eq!(SnapshotPlanDayOfWeek::new(8), None);
    assert_eq!(SnapshotPlanDayOfMonth::new(32), None);

    let max_snapshots = SnapshotPlanMaxSnapshots::new(5);
    assert!(max_snapshots.is_some(), "fixture max validates");
    let Some(max_snapshots) = max_snapshots else {
        return;
    };
    let minute = SnapshotPlanMinute::new(30);
    assert!(minute.is_some(), "fixture minute validates");
    let Some(minute) = minute else { return };
    let hour = SnapshotPlanHour::new(3);
    assert!(hour.is_some(), "fixture hour validates");
    let Some(hour) = hour else { return };
    assert_eq!(
        StorageBoxSnapshotPlanRequest::try_new(Some(max_snapshots), Some(minute), Some(hour))
            .map(StorageBoxSnapshotPlanRequest::max_snapshots),
        Ok(max_snapshots)
    );
    assert_eq!(
        StorageBoxRollbackSnapshotRequest::try_new(Some(snapshot))
            .map(StorageBoxRollbackSnapshotRequest::snapshot),
        Ok(snapshot)
    );

    assert_eq!(
        StorageBoxHomeDirectory::new("/absolute"),
        Err(StorageBoxRequestError::InvalidText)
    );
    assert_eq!(
        StorageBoxHomeDirectory::new("safe/../escape"),
        Err(StorageBoxRequestError::InvalidText)
    );
    let home = StorageBoxHomeDirectory::new("safe/path");
    assert!(home.is_ok(), "fixture home dir validates");
    let Ok(home) = home else { return };
    let password = StorageBoxPassword::new("not-logged");
    assert!(password.is_ok(), "fixture password validates");
    let Ok(password) = password else { return };
    assert_eq!(
        StorageBoxSubaccountCreateRequest::try_new(id, Some(home), Some(password))
            .map(StorageBoxSubaccountCreateRequest::endpoint),
        Ok(StorageBoxSubaccountEndpoint::Create(id))
    );
    assert_eq!(
        StorageBoxChangeHomeDirectoryRequest::try_new(Some(home))
            .map(StorageBoxChangeHomeDirectoryRequest::home_directory),
        Ok(home)
    );
}

fn storage_box_id() -> Option<StorageBoxId> {
    StorageBoxId::new(42)
}

fn assert_path(result: Result<usize, StorageBoxRequestError>, output: &[u8], expected: &str) {
    assert_query(result, output, expected);
}

fn assert_query(result: Result<usize, StorageBoxRequestError>, output: &[u8], expected: &str) {
    assert!(result.is_ok(), "write must succeed");
    let Ok(len) = result else { return };
    let actual = output
        .get(..len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok());
    assert_eq!(actual, Some(expected));
}
