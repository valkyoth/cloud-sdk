#![no_main]

use cloud_sdk::rate_limit::QuotaReset;
use cloud_sdk::transport::{
    HeaderSensitivity, ResponseBuffer, ResponseDecodeWorkspace, ResponseMetadata, StatusCode,
};
use cloud_sdk_hetzner::robot::{
    MAX_ROBOT_ERROR_BODY_BYTES, RobotFailure, RobotRetryDisposition, decode_robot_failure,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let status = match data.first().copied().unwrap_or(0) % 7 {
        0 => 400,
        1 => 401,
        2 => 403,
        3 => 404,
        4 => 503,
        5 => 200,
        _ => 500,
    };
    let body = data.get(1..).unwrap_or_default();
    if body.len() > MAX_ROBOT_ERROR_BODY_BYTES {
        return;
    }
    let mut storage = body.to_vec();
    let mut header_storage = [0_u8; 256];
    let capacity = storage.len();
    let mut buffer = ResponseBuffer::new(&mut storage, capacity, &mut header_storage);
    let mut attempt = buffer
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!());
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!())
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!());
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!())
        .copy_from_slice(body);
    attempt
        .commit(
            StatusCode::new(status).unwrap_or_else(|| unreachable!()),
            body.len(),
            ResponseMetadata::EMPTY,
        )
        .unwrap_or_else(|_| unreachable!());
    drop(attempt);

    let mut workspace = ResponseDecodeWorkspace::new_for_provider();
    let decoded = buffer
        .with_response(|response| decode_robot_failure(response, &mut workspace))
        .unwrap_or_else(|_| unreachable!());
    if let Ok(failure) = decoded {
        assert!(!failure.allows_automatic_retry());
        match failure {
            RobotFailure::AuthenticationRejected => {
                assert_eq!(status, 401);
                assert_eq!(
                    RobotFailure::AuthenticationRejected.retry_disposition(),
                    RobotRetryDisposition::Never
                );
            }
            RobotFailure::Maintenance => assert_eq!(status, 503),
            RobotFailure::InvalidInput(_) => assert_eq!(status, 400),
            RobotFailure::QuotaExceeded(quota) => {
                assert_eq!(status, 403);
                assert_ne!(quota.max_requests(), 0);
                assert_ne!(quota.interval().get(), 0);
                let bucket = quota.quota_bucket().unwrap_or_else(|_| unreachable!());
                assert!(matches!(bucket.reset(), QuotaReset::After(_)));
            }
            RobotFailure::Provider(_) => assert_eq!(status, 404),
            RobotFailure::TransientTransport(_) => {
                unreachable!("provider bytes created a transport classification")
            }
        }
    }
});
