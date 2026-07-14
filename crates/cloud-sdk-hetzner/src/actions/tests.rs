use cloud_sdk::Method;

use super::{
    ActionEndpoint, ActionId, ActionListRequest, ActionRequestError, ActionStatus,
    MAX_ACTION_FILTER_IDS, MAX_ACTION_ID,
};
use crate::{EndpointGroup, request::ApiBaseUrl};

#[test]
fn parses_action_status() {
    assert_eq!(
        ActionStatus::from_api_str("running"),
        Some(ActionStatus::Running)
    );
    assert_eq!(ActionStatus::from_api_str("unknown"), None);
    assert!(!ActionStatus::Running.is_terminal());
    assert!(ActionStatus::Success.is_terminal());
    assert!(ActionStatus::Error.is_terminal());
}

#[test]
fn action_ids_enforce_source_locked_integer_bounds() {
    assert_eq!(ActionId::new(0), None);
    assert_eq!(
        ActionId::new(MAX_ACTION_ID).map(ActionId::get),
        Some(MAX_ACTION_ID)
    );
    assert_eq!(ActionId::new(MAX_ACTION_ID + 1), None);
}

#[test]
fn global_action_paths_and_metadata_match_source_lock() -> Result<(), ActionRequestError> {
    let id = ActionId::new(42).ok_or(ActionRequestError::EmptyActionIds)?;
    let mut output = [0_u8; 64];

    let len = ActionEndpoint::List.write_path(&mut output)?;
    assert_eq!(output.get(..len), Some(b"/actions".as_slice()));
    assert_eq!(ActionEndpoint::List.method(), Method::Get);
    assert_eq!(ActionEndpoint::List.api_base_url(), ApiBaseUrl::CloudV1);
    assert_eq!(
        ActionEndpoint::List.endpoint_group(),
        EndpointGroup::Actions
    );
    let static_path = ActionEndpoint::List
        .static_path()
        .ok_or(ActionRequestError::EmptyActionIds)??;
    assert_eq!(static_path.as_str(), "/actions");

    let len = ActionEndpoint::Get(id).write_path(&mut output)?;
    assert_eq!(output.get(..len), Some(b"/actions/42".as_slice()));
    assert!(ActionEndpoint::Get(id).static_path().is_none());
    Ok(())
}

#[test]
fn global_action_list_requires_a_bounded_id_filter() -> Result<(), ActionRequestError> {
    assert_eq!(
        ActionListRequest::try_new(&[]),
        Err(ActionRequestError::EmptyActionIds)
    );

    let id = ActionId::new(1).ok_or(ActionRequestError::EmptyActionIds)?;
    let ids = [id; MAX_ACTION_FILTER_IDS + 1];
    assert_eq!(
        ActionListRequest::try_new(&ids),
        Err(ActionRequestError::TooManyActionIds)
    );
    Ok(())
}

#[test]
fn global_action_list_writes_repeated_ids_in_caller_order() -> Result<(), ActionRequestError> {
    let ids = [
        ActionId::new(42).ok_or(ActionRequestError::EmptyActionIds)?,
        ActionId::new(MAX_ACTION_ID).ok_or(ActionRequestError::EmptyActionIds)?,
        ActionId::new(7).ok_or(ActionRequestError::EmptyActionIds)?,
    ];
    let request = ActionListRequest::try_new(&ids)?;
    let mut output = [0_u8; 96];
    let len = request.write_query(&mut output)?;

    assert_eq!(request.endpoint(), ActionEndpoint::List);
    assert_eq!(request.ids(), ids);
    assert_eq!(
        output.get(..len),
        Some(b"id=42&id=9007199254740991&id=7".as_slice())
    );
    Ok(())
}

#[test]
fn global_action_paths_and_queries_fail_closed_on_small_buffers() -> Result<(), ActionRequestError>
{
    let id = ActionId::new(42).ok_or(ActionRequestError::EmptyActionIds)?;
    let request = ActionListRequest::try_new(core::slice::from_ref(&id))?;

    assert_eq!(
        ActionEndpoint::Get(id).write_path(&mut [0_u8; 10]),
        Err(ActionRequestError::PathBufferTooSmall)
    );
    assert_eq!(
        request.write_query(&mut [0_u8; 4]),
        Err(ActionRequestError::QueryBufferTooSmall)
    );
    Ok(())
}
