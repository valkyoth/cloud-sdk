use alloc::{format, vec};

use cloud_sdk::operation::PreparationStorageGuard;
use cloud_sdk::transport::{
    HeaderSensitivity, ResponseBuffer, ResponseDecodeWorkspace, ResponseMetadata, StatusCode,
    TransportResponse,
};

use super::basic::STANDARD_CREATED;
use super::*;
use crate::robot::ordering::RobotStandardTransactionList;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode};

#[test]
fn mismatched_success_and_matching_history_fail_closed() {
    with_standard_plan(|plan| {
        let request = RobotStandardOrderCreateRequest::new(plan, 1_100_000)
            .unwrap_or_else(|_| unreachable!("standard request failed"));

        let mismatched = core::str::from_utf8(STANDARD_CREATED)
            .unwrap_or_else(|_| unreachable!("fixture is not UTF-8"))
            .replace("EX40", "EX41");
        let mut target = [0_u8; 128];
        let mut request_body = [0_u8; 256];
        let mut guard = PreparationStorageGuard::new(&mut target, &mut request_body);
        let prepared = prepared_standard(&request, &mut guard);
        let mut body = vec![0_u8; mismatched.len()];
        let mut headers = [0_u8; 128];
        let response = json_response(
            &mut body,
            &mut headers,
            StatusCode::CREATED,
            mismatched.as_bytes(),
        );
        assert_eq!(
            prepared
                .validate_response(response)
                .unwrap_or_else(|_| unreachable!("response policy failed"))
                .decode_response()
                .err(),
            Some(RobotOrderMutationDecodeError::ResponseIntentMismatch)
        );

        let mut target = [0_u8; 128];
        let mut request_body = [0_u8; 256];
        let mut guard = PreparationStorageGuard::new(&mut target, &mut request_body);
        let prepared = prepared_standard(&request, &mut guard);
        let mut body = [0_u8; STANDARD_CREATED.len()];
        let mut headers = [0_u8; 128];
        let response = json_response(
            &mut body,
            &mut headers,
            StatusCode::CREATED,
            STANDARD_CREATED,
        );
        let transaction = prepared
            .validate_response(response)
            .unwrap_or_else(|_| unreachable!("response policy failed"))
            .decode_response()
            .unwrap_or_else(|_| unreachable!("transaction fixture failed"));
        let history = RobotStandardTransactionList(vec![transaction]);
        assert_eq!(
            request.reconcile_not_applied(&history).err(),
            Some(RobotOrderReconciliationError::MatchingTransaction)
        );
    });
}

#[test]
fn failed_preparation_clears_all_guarded_storage() {
    with_standard_plan(|plan| {
        let request = RobotStandardOrderCreateRequest::new(plan, 1_100_000)
            .unwrap_or_else(|_| unreachable!("standard request failed"));
        let mut target = [0xa5_u8; 8];
        let mut body = [0x5a_u8; 8];
        {
            let mut guard = PreparationStorageGuard::new(&mut target, &mut body);
            assert!(request.prepare_bound(&mut guard).is_err());
        }
        assert_eq!(target, [0_u8; 8]);
        assert_eq!(body, [0_u8; 8]);
    });
}

#[test]
fn source_locked_failures_are_family_bound() {
    with_standard_plan(|plan| {
        let request = RobotStandardOrderCreateRequest::new(plan, 1_100_000)
            .unwrap_or_else(|_| unreachable!("standard request failed"));
        assert_provider_failure(
            &request,
            412,
            "PRECONDITION_FAILED",
            RobotProviderErrorCode::OrderPreconditionFailed,
        );
        assert_provider_failure(
            &request,
            500,
            "INTERNAL_ERROR",
            RobotProviderErrorCode::OrderInternalError,
        );
        assert_eq!(
            decode_failure(&request, 409, "CONFLICT").err(),
            Some(RobotDecodeError::UnsupportedStatus)
        );
    });
    with_addon_plan(|plan| {
        let request = RobotAddonOrderCreateRequest::new(plan, 300_000)
            .unwrap_or_else(|_| unreachable!("addon request failed"));
        assert_provider_failure(
            &request,
            409,
            "CONFLICT",
            RobotProviderErrorCode::OrderConflict,
        );
    });
}

trait DecodeOrderFailure {
    fn decode<'a>(
        &self,
        response: TransportResponse<'a, 'a>,
        workspace: &mut ResponseDecodeWorkspace,
    ) -> Result<RobotFailure, RobotDecodeError>;
}

impl DecodeOrderFailure for RobotStandardOrderCreateRequest<'_> {
    fn decode<'a>(
        &self,
        response: TransportResponse<'a, 'a>,
        workspace: &mut ResponseDecodeWorkspace,
    ) -> Result<RobotFailure, RobotDecodeError> {
        self.decode_failure(response, workspace)
    }
}

impl DecodeOrderFailure for RobotAddonOrderCreateRequest<'_, '_> {
    fn decode<'a>(
        &self,
        response: TransportResponse<'a, 'a>,
        workspace: &mut ResponseDecodeWorkspace,
    ) -> Result<RobotFailure, RobotDecodeError> {
        self.decode_failure(response, workspace)
    }
}

fn assert_provider_failure<R: DecodeOrderFailure>(
    request: &R,
    status: u16,
    code: &str,
    expected: RobotProviderErrorCode,
) {
    let failure = decode_failure(request, status, code)
        .unwrap_or_else(|_| unreachable!("source-locked failure rejected"));
    let RobotFailure::Provider(provider) = failure else {
        unreachable!("provider failure changed category")
    };
    assert_eq!(provider.code(), expected);
}

fn decode_failure<R: DecodeOrderFailure>(
    request: &R,
    status: u16,
    code: &str,
) -> Result<RobotFailure, RobotDecodeError> {
    let wire = format!(r#"{{"error":{{"status":{status},"code":"{code}","message":"redacted"}}}}"#);
    let mut body = vec![0_u8; wire.len()];
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut body, wire.len(), &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("failure response attempt failed"));
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!("failure response headers failed"))
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!("failure content type failed"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("failure response body failed"))
        .copy_from_slice(wire.as_bytes());
    attempt
        .commit(
            StatusCode::new(status).unwrap_or_else(|| unreachable!("invalid fixture status")),
            wire.len(),
            ResponseMetadata::EMPTY,
        )
        .unwrap_or_else(|_| unreachable!("failure response commit failed"));
    drop(attempt);
    let mut workspace = ResponseDecodeWorkspace::new_for_provider();
    response
        .with_response(|response| request.decode(response, &mut workspace))
        .unwrap_or_else(|_| unreachable!("committed failure response unavailable"))
}
