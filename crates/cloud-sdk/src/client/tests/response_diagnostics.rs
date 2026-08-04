use super::fixture::ExampleOperation;
use crate::client::ClientResponse;
use crate::diagnostics::DiagnosticRequestId;
use crate::operation::{PreparationStorage, PrepareOperation, RequestIdPolicy};
use crate::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

#[test]
fn diagnostics_classify_raw_request_id_presence_for_every_policy() {
    for (policy, expected_present, expected_absent) in [
        (
            RequestIdPolicy::Discard,
            DiagnosticRequestId::Discarded,
            DiagnosticRequestId::Discarded,
        ),
        (
            RequestIdPolicy::Protected,
            DiagnosticRequestId::Protected,
            DiagnosticRequestId::Absent,
        ),
        (
            RequestIdPolicy::Retain,
            DiagnosticRequestId::Retainable,
            DiagnosticRequestId::Absent,
        ),
    ] {
        assert_eq!(request_id_diagnostic(policy, true), Some(expected_present));
        assert_eq!(request_id_diagnostic(policy, false), Some(expected_absent));
    }
}

fn request_id_diagnostic(
    policy: RequestIdPolicy,
    include_request_id: bool,
) -> Option<DiagnosticRequestId> {
    let mut target = [0_u8; 16];
    let mut request_body = [0_u8; 16];
    let operation = ExampleOperation::read_only().with_request_id_policy(policy);
    let prepared = operation
        .prepare(PreparationStorage::new(&mut target, &mut request_body))
        .ok()?;
    let mut body = [0_u8; 8];
    let mut header_storage = [0_u8; 64];
    let mut response = ResponseBuffer::new(&mut body, 8, &mut header_storage);
    let mut attempt = response.writer().begin_attempt().ok()?;
    if include_request_id {
        attempt
            .staging()
            .headers_mut()
            .ok()?
            .try_push("x-request-id", b"request-123", HeaderSensitivity::Public)
            .ok()?;
    }
    attempt
        .commit(StatusCode::OK, 0, ResponseMetadata::EMPTY)
        .ok()?;
    drop(attempt);
    ClientResponse::new(prepared, response)
        .diagnostic_response()
        .ok()
        .map(|diagnostic| diagnostic.request_id())
}
