use cloud_sdk::authentication::ScopeRequirement;
use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk::transport::{
    HeaderSensitivity, ResponseBuffer, ResponseHeaders, ResponseMetadata, StatusCode,
};
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, MockExchange, MockTransport, ResponseFixture,
};

use crate::actions::{ActionEndpoint, ActionId};

#[test]
fn successful_prepared_execution_exposes_the_protected_request_id() -> Result<(), &'static str> {
    let id = ActionId::new(7).ok_or("action ID")?;
    let operation = ActionEndpoint::Get(id);
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 1];
    let prepared = operation
        .prepare(PreparationStorage::new(&mut target, &mut request_body))
        .map_err(|_| "preparation")?;
    let request = prepared.transport_request();
    let expected = ExpectedRequest::new(request.method(), request.target())
        .with_body(request.body())
        .with_headers(request.headers());
    let body = FixtureBody::new(b"{}").map_err(|_| "fixture body")?;
    let mut fixture_header_storage = [0_u8; 256];
    let mut fixture_headers = ResponseHeaders::new(&mut fixture_header_storage);
    fixture_headers
        .try_push(
            "x-request-id",
            b"success-request-123",
            HeaderSensitivity::Sensitive,
        )
        .map_err(|_| "fixture request ID")?;
    let fixture = ResponseFixture::success(body)
        .with_content_type("application/json")
        .with_headers(fixture_headers);
    let exchange = MockExchange::new(expected, fixture);
    let ScopeRequirement::Required(endpoint) =
        prepared.authentication_policy().endpoint_requirement()
    else {
        return Err("official endpoint policy");
    };
    let exchanges = [exchange];
    let transport = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut response_body = [0_u8; 32];
    let mut response_header_storage = [0_u8; 512];
    let checked = prepared
        .execute_blocking(&transport, &mut response_body, &mut response_header_storage)
        .map_err(|_| "prepared execution")?;
    assert!(checked.with_borrowed(|response| {
        response.with_request_id(|value| value == Some(b"success-request-123".as_slice()))
    }));
    Ok(())
}

#[test]
fn provider_error_metadata_exposes_the_protected_request_id() -> Result<(), &'static str> {
    let id = ActionId::new(7).ok_or("action ID")?;
    let operation = ActionEndpoint::Get(id);
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 1];
    let prepared = operation
        .prepare(PreparationStorage::new(&mut target, &mut request_body))
        .map_err(|_| "preparation")?;
    let mut response_body = [0_u8; 128];
    let mut response_header_storage = [0_u8; 512];
    let mut response = ResponseBuffer::new(
        &mut response_body,
        prepared.raw_response_policy().max_body_bytes(),
        &mut response_header_storage,
    );
    let mut attempt = response.writer().begin_attempt().map_err(|_| "attempt")?;
    attempt
        .headers_mut()
        .map_err(|_| "headers")?
        .try_push(
            "x-request-id",
            b"error-request-456",
            HeaderSensitivity::Sensitive,
        )
        .map_err(|_| "request ID")?;
    let body = br#"{"error":{"code":"rate_limit","message":"wait"}}"#;
    attempt
        .body_mut()
        .map_err(|_| "body")?
        .get_mut(..body.len())
        .ok_or("body capacity")?
        .copy_from_slice(body);
    attempt
        .commit(
            StatusCode::TOO_MANY_REQUESTS,
            body.len(),
            ResponseMetadata::EMPTY,
        )
        .map_err(|_| "commit")?;
    drop(attempt);
    prepared
        .apply_response_metadata_policy(&mut response)
        .map_err(|_| "metadata policy")?;
    let protected = response
        .with_response(|view| {
            view.with_request_id(|value| value == Some(b"error-request-456".as_slice()))
        })
        .map_err(|_| "response view")?;
    assert!(protected);
    Ok(())
}
