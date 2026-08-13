#![no_main]

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    CheckedRobotBoot, PreparedRobotBoot, RobotBootGetRequest, RobotBootKey, RobotBootValue,
    RobotLinuxActivateRequest, RobotLinuxGetRequest, RobotRescueActivateRequest,
    RobotRescueGetRequest, RobotServerNumber, RobotVncActivateRequest, RobotVncGetRequest,
    RobotWindowsActivateRequest, RobotWindowsGetRequest,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((&selector, body)) = data.split_first() else {
        return;
    };
    match selector % 9 {
        0 => overview(body),
        1 => rescue(body),
        2 => linux(body),
        3 => vnc(body),
        4 => windows(body),
        5 => rescue_activation(body),
        6 => linux_activation(body),
        7 => vnc_activation(body),
        _ => windows_activation(body),
    }
});

macro_rules! read_case {
    ($name:ident, $request:expr) => {
        fn $name(body: &[u8]) {
            let request = $request;
            let mut target = [0_u8; 96];
            let mut request_body = [0_u8; 512];
            let prepared = request
                .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
                .unwrap_or_else(|_| unreachable!("fixed boot read failed"));
            run(prepared, body, |checked| checked.decode_response());
        }
    };
}

read_case!(overview, RobotBootGetRequest::new(number()));
read_case!(rescue, RobotRescueGetRequest::new(number()));
read_case!(linux, RobotLinuxGetRequest::new(number()));
read_case!(vnc, RobotVncGetRequest::new(number()));
read_case!(windows, RobotWindowsGetRequest::new(number()));

fn rescue_activation(body: &[u8]) {
    let keys = [RobotBootKey::new("SHA256:key").unwrap_or_else(|_| unreachable!())];
    let request = RobotRescueActivateRequest::new(number(), value("linux"), &keys, None)
        .unwrap_or_else(|_| unreachable!());
    let mut target = [0_u8; 96];
    let mut request_body = [0_u8; 512];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed Rescue activation failed"));
    run(prepared, body, |checked| checked.decode_response());
}

fn linux_activation(body: &[u8]) {
    let request = RobotLinuxActivateRequest::new(number(), value("Debian 13"), value("en"), &[])
        .unwrap_or_else(|_| unreachable!());
    let mut target = [0_u8; 96];
    let mut request_body = [0_u8; 512];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed Linux activation failed"));
    run(prepared, body, |checked| checked.decode_response());
}

fn vnc_activation(body: &[u8]) {
    let request = RobotVncActivateRequest::new(number(), value("rescue-vnc"), value("en_US"));
    let mut target = [0_u8; 96];
    let mut request_body = [0_u8; 512];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed VNC activation failed"));
    run(prepared, body, |checked| checked.decode_response());
}

fn windows_activation(body: &[u8]) {
    let request = RobotWindowsActivateRequest::new(
        number(),
        value("Windows Server 2022 Standard Edition"),
        value("en"),
    );
    let mut target = [0_u8; 96];
    let mut request_body = [0_u8; 512];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed Windows activation failed"));
    run(prepared, body, |checked| checked.decode_response());
}

fn run<R, O>(
    prepared: PreparedRobotBoot<'_, '_, R>,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotBoot<'_, '_, R>) -> O,
) {
    let mut response_storage = body.to_vec();
    let capacity = response_storage.len();
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut response_storage, capacity, &mut headers);
    write_response(&mut response, body);
    if let Ok(checked) = prepared.validate_response(response) {
        let _ = decode(checked);
    }
}

fn write_response(response: &mut ResponseBuffer<'_>, body: &[u8]) {
    let mut attempt = response
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
        .commit(StatusCode::OK, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!());
}

fn number() -> RobotServerNumber {
    RobotServerNumber::new(321).unwrap_or_else(|_| unreachable!())
}
fn value(value: &str) -> RobotBootValue<'_> {
    RobotBootValue::new(value).unwrap_or_else(|_| unreachable!())
}
