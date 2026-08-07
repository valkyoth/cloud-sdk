use alloc::format;

use cloud_sdk::transport::StatusCode;

use super::checked_test_support::{decode_response, prepared, response};
use super::{HetznerDecodeError, ResponseModelError};
use crate::identity::CLOUD_SERVICE_ID;

fn action() -> &'static str {
    r#"{"id":42,"command":"poweron_server","status":"running","progress":10,"started":"2026-07-16T00:00:00Z","finished":null,"resources":[{"id":7,"type":"server"}],"error":null}"#
}

fn one_item_page() -> &'static str {
    r#"{"pagination":{"page":1,"per_page":1,"previous_page":null,"next_page":2,"last_page":2,"total_entries":2}}"#
}

#[test]
fn numbered_action_and_resource_pages_reject_more_items_than_per_page() {
    let actions = format!(
        r#"{{"actions":[{0},{0}],"meta":{1}}}"#,
        action(),
        one_item_page(),
    );
    assert_eq!(
        decode_response(
            prepared("list_servers_actions", CLOUD_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, actions.as_bytes()),
        ),
        Err(HetznerDecodeError::Model(
            ResponseModelError::InvalidPagination,
        )),
    );

    let resources = format!(
        r#"{{"servers":[{{"id":1,"name":"one","status":"running"}},{{"id":2,"name":"two","status":"running"}}],"meta":{}}}"#,
        one_item_page(),
    );
    assert_eq!(
        decode_response(
            prepared("list_servers", CLOUD_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, resources.as_bytes()),
        ),
        Err(HetznerDecodeError::Model(
            ResponseModelError::InvalidPagination,
        )),
    );
}
