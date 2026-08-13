#![no_main]

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    RobotSshKeyCreateRequest, RobotSshKeyData, RobotSshKeyFingerprint, RobotSshKeyGetRequest,
    RobotSshKeyListRequest, RobotSshKeyName,
};
use libfuzzer_sys::fuzz_target;

const FINGERPRINT: &str = "ae:6f:ba:1b:70:2c:ae:c7:5c:ab:6e:4d:5e:d4:c7:23";
const SSH2_KEY: &str = "---- BEGIN SSH2 PUBLIC KEY ----\nComment: fuzz\nAAAAC3NzaC1lZDI1NTE5AAAAILM+rvN+ot98qgEN796jTiQfZfG1KaT0PtFDJ/XFSqti\n---- END SSH2 PUBLIC KEY ----";

fuzz_target!(|input: &[u8]| {
    let Some((&selector, body)) = input.split_first() else {
        return;
    };
    match selector % 3 {
        0 => fuzz_get_response(body),
        1 => fuzz_list_response(body),
        _ => fuzz_create_response(body),
    }
});

fn fuzz_get_response(body: &[u8]) {
    let fingerprint = RobotSshKeyFingerprint::new(FINGERPRINT)
        .unwrap_or_else(|_| unreachable!("fixed SSH-key fingerprint failed"));
    let request = RobotSshKeyGetRequest::new(fingerprint);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed SSH-key get preparation failed"));
    with_response(StatusCode::OK, body, |response| {
        if let Ok(checked) = prepared.validate_response(response) {
            let _ = checked.decode_response();
        }
    });
}

fn fuzz_list_response(body: &[u8]) {
    let request = RobotSshKeyListRequest::new();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed SSH-key list preparation failed"));
    with_response(StatusCode::OK, body, |response| {
        if let Ok(checked) = prepared.validate_response(response) {
            let _ = checked.decode_response();
        }
    });
}

fn fuzz_create_response(body: &[u8]) {
    let name = RobotSshKeyName::new("deploy-key")
        .unwrap_or_else(|_| unreachable!("fixed SSH-key name failed"));
    let data =
        RobotSshKeyData::new(SSH2_KEY).unwrap_or_else(|_| unreachable!("fixed SSH2 key failed"));
    let request = RobotSshKeyCreateRequest::new(name, data);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 512];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed SSH-key create preparation failed"));
    with_response(StatusCode::CREATED, body, |response| {
        if let Ok(checked) = prepared.validate_response(response) {
            let _ = checked.decode_response();
        }
    });
}

fn with_response(body_status: StatusCode, body: &[u8], inspect: impl FnOnce(ResponseBuffer<'_>)) {
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
        .commit(body_status, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("SSH-key response commit failed"));
    drop(attempt);
    inspect(response);
}
