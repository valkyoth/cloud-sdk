//! Credential-free end-to-end execution matrix for the source-locked probe.

mod support;

use core::future::Future;
use core::task::{Context, Poll, Waker};

use cloud_sdk::operation::{OperationImpact, RequestSemantics, RetryEligibility};
use cloud_sdk::transport::EndpointPolicy;
use cloud_sdk_testkit::{LocalMockTransport, MockTransport, PreparedRequestRecord};

use support::{OPERATIONS, Operation, endpoint, exchange, prepared, request_headers};

#[test]
fn catalog_matches_the_source_locked_candidate_inventory() {
    let inventory = include_str!("../../CANDIDATES.tsv");
    let mut rows = inventory.lines();
    assert_eq!(
        rows.next(),
        Some("id\tmethod\tpath\tpagination\tresponse_type\tstability")
    );
    for operation in OPERATIONS {
        let Some(row) = rows.next() else {
            unreachable!("candidate inventory lost a source-locked operation");
        };
        let mut fields = row.split('\t');
        assert_eq!(fields.next(), Some(operation.id));
        assert_eq!(fields.next(), Some("GET"));
        assert_eq!(fields.next(), Some(operation.template));
        assert_eq!(
            fields.next(),
            Some(if operation.paginated {
                "cursor"
            } else {
                "none"
            })
        );
        assert!(fields.next().is_some());
        assert_eq!(fields.next(), Some("production"));
        assert_eq!(fields.next(), None);
    }
    assert_eq!(rows.next(), None);
}

#[test]
fn every_operation_executes_through_all_neutral_contracts() {
    for operation in OPERATIONS {
        execute_blocking(operation);
        execute_send_async(operation);
        execute_local_async(operation);
    }
}

#[test]
fn every_operation_is_fail_closed_read_only_and_non_retrying() {
    for operation in OPERATIONS {
        let (headers, count) = request_headers(operation.paginated);
        let Some(entries) = headers.get(..count) else {
            unreachable!("pagination header count exceeds fixture storage");
        };
        let prepared = prepared(operation, entries);
        let record = PreparedRequestRecord::capture(prepared);
        assert_eq!(record.metadata().impact(), OperationImpact::ReadOnly);
        assert_eq!(record.metadata().semantics(), RequestSemantics::Safe);
        assert_eq!(
            record.metadata().retry_eligibility(),
            RetryEligibility::Never
        );
        assert_eq!(record.body_len(), 0);
        assert_eq!(record.header_count(), count);
        assert_eq!(record.sensitive_header_count(), 0);
        assert!(matches!(
            record.service().endpoint_policy(),
            EndpointPolicy::Fixed(identity) if identity == endpoint()
        ));
    }
}

fn execute_blocking(operation: Operation) {
    let (headers, count) = request_headers(operation.paginated);
    let Some(entries) = headers.get(..count) else {
        unreachable!("pagination header count exceeds fixture storage");
    };
    let prepared = prepared(operation, entries);
    let exchanges = [exchange(operation, entries)];
    let mock = MockTransport::new(&exchanges).with_endpoint(endpoint());
    let mut body = [0_u8; 65_536];
    let mut response_headers = [0_u8; 8192];
    let response = prepared.execute_blocking(&mock, &mut body, &mut response_headers);
    assert!(response.is_ok());
    let Ok(response) = response else {
        unreachable!("credential-free blocking fixture must execute");
    };
    assert!(response.with_borrowed(|checked| checked.body() == operation.response));
    assert!(mock.is_complete());
}

fn execute_send_async(operation: Operation) {
    let (headers, count) = request_headers(operation.paginated);
    let Some(entries) = headers.get(..count) else {
        unreachable!("pagination header count exceeds fixture storage");
    };
    let prepared = prepared(operation, entries);
    let exchanges = [exchange(operation, entries)];
    let mock = MockTransport::new(&exchanges).with_endpoint(endpoint());
    let mut body = [0_u8; 65_536];
    let mut response_headers = [0_u8; 8192];
    let future = prepared.execute_async(&mock, &mut body, &mut response_headers);
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    let result = Future::poll(future.as_mut(), &mut context);
    assert!(matches!(result, Poll::Ready(Ok(_))));
    assert!(mock.is_complete());
}

fn execute_local_async(operation: Operation) {
    let (headers, count) = request_headers(operation.paginated);
    let Some(entries) = headers.get(..count) else {
        unreachable!("pagination header count exceeds fixture storage");
    };
    let prepared = prepared(operation, entries);
    let exchanges = [exchange(operation, entries)];
    let mock = LocalMockTransport::new(&exchanges).with_endpoint(endpoint());
    let mut body = [0_u8; 65_536];
    let mut response_headers = [0_u8; 8192];
    let future = prepared.execute_local_async(&mock, &mut body, &mut response_headers);
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    let result = Future::poll(future.as_mut(), &mut context);
    assert!(matches!(result, Poll::Ready(Ok(_))));
    assert!(mock.is_complete());
}
