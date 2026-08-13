#![no_main]

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{RobotSshKeyFingerprint, RobotSshKeyGetRequest};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|body: &[u8]| {
    let fingerprint =
        RobotSshKeyFingerprint::new("ae:6f:ba:1b:70:2c:ae:c7:5c:ab:6e:4d:5e:d4:c7:23")
            .unwrap_or_else(|_| unreachable!("fixed SSH-key fingerprint failed"));
    let request = RobotSshKeyGetRequest::new(fingerprint);
    let mut target_storage = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(
            &mut target_storage,
            &mut request_body,
        ))
        .unwrap_or_else(|_| unreachable!("fixed SSH-key preparation failed"));

    let mut response_storage = body.to_vec();
    let capacity = response_storage.len();
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut response_storage, capacity, &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("SSH-key response attempt failed"));
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!("SSH-key response headers failed"))
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!("SSH-key content type failed"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("SSH-key response body failed"))
        .copy_from_slice(body);
    attempt
        .commit(StatusCode::OK, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("SSH-key response commit failed"));
    drop(attempt);
    if let Ok(checked) = prepared.validate_response(response) {
        let _ = checked.decode_response();
    }
});
