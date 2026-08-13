use alloc::{format, vec};

use cloud_sdk::Method;
use cloud_sdk::operation::{
    OperationImpact, PreparationStorage, PrepareOperation, RequestBodySensitivity,
    RequestSemantics, RetryEligibility,
};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::*;
use crate::robot::RobotServerNumber;

const RESCUE_ACTIVE: &[u8] = br#"{"rescue":{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8::","server_number":321,"os":"linux","@deprecated arch":64,"active":true,"password":"generated-password","authorized_key":["SHA256:key"],"host_key":["ssh-ed25519 host"]}}"#;
const LINUX_INACTIVE: &[u8] = br#"{"linux":{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8::","server_number":321,"dist":["Debian 13 minimal"],"@deprecated arch":[64],"lang":["en"],"active":false,"password":null,"authorized_key":[],"host_key":[]}}"#;
const VNC_INACTIVE: &[u8] = br#"{"vnc":{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8::","server_number":321,"dist":["rescue-vnc"],"lang":["en_US"],"active":false,"password":null}}"#;
const WINDOWS_INACTIVE: &[u8] = br#"{"windows":{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8::","server_number":321,"dist":["standard"],"os":["Windows Server 2022 Standard Edition"],"lang":["en"],"active":false,"password":null}}"#;

#[test]
fn prepares_all_fifteen_canonical_operations() {
    macro_rules! read_case {
        ($request:expr, $method:expr, $path:literal, $id:literal, $impact:expr) => {{
            let request = $request;
            let mut target = [0_u8; 96];
            let mut body = [0_u8; 512];
            let prepared = request
                .prepare(PreparationStorage::new(&mut target, &mut body))
                .unwrap_or_else(|_| unreachable!("boot preparation failed"));
            assert_prepared(prepared, $method, $path, $id, $impact, false);
        }};
    }
    read_case!(
        RobotBootGetRequest::new(number()),
        Method::Get,
        "/boot/321",
        "robot_get_boot",
        OperationImpact::ReadOnly
    );
    read_case!(
        RobotRescueGetRequest::new(number()),
        Method::Get,
        "/boot/321/rescue",
        "robot_get_rescue",
        OperationImpact::ReadOnly
    );
    read_case!(
        RobotRescueDeactivateRequest::new(number()),
        Method::Delete,
        "/boot/321/rescue",
        "robot_deactivate_rescue",
        OperationImpact::Mutation
    );
    read_case!(
        RobotRescueLastRequest::new(number()),
        Method::Get,
        "/boot/321/rescue/last",
        "robot_get_last_rescue",
        OperationImpact::ReadOnly
    );
    read_case!(
        RobotLinuxGetRequest::new(number()),
        Method::Get,
        "/boot/321/linux",
        "robot_get_linux",
        OperationImpact::ReadOnly
    );
    read_case!(
        RobotLinuxDeactivateRequest::new(number()),
        Method::Delete,
        "/boot/321/linux",
        "robot_deactivate_linux",
        OperationImpact::Mutation
    );
    read_case!(
        RobotLinuxLastRequest::new(number()),
        Method::Get,
        "/boot/321/linux/last",
        "robot_get_last_linux",
        OperationImpact::ReadOnly
    );
    read_case!(
        RobotVncGetRequest::new(number()),
        Method::Get,
        "/boot/321/vnc",
        "robot_get_vnc",
        OperationImpact::ReadOnly
    );
    read_case!(
        RobotVncDeactivateRequest::new(number()),
        Method::Delete,
        "/boot/321/vnc",
        "robot_deactivate_vnc",
        OperationImpact::Mutation
    );
    read_case!(
        RobotWindowsGetRequest::new(number()),
        Method::Get,
        "/boot/321/windows",
        "robot_get_windows",
        OperationImpact::ReadOnly
    );
    read_case!(
        RobotWindowsDeactivateRequest::new(number()),
        Method::Delete,
        "/boot/321/windows",
        "robot_deactivate_windows",
        OperationImpact::Mutation
    );

    let os = value("linux");
    let keys = [RobotBootKey::new("SHA256:key one").unwrap_or_else(|_| unreachable!())];
    let rescue = RobotRescueActivateRequest::new(number(), os, &keys, Some(keyboard("de")))
        .unwrap_or_else(|_| unreachable!());
    assert_activation(
        &rescue,
        "/boot/321/rescue",
        "robot_activate_rescue",
        OperationImpact::Mutation,
        b"os=linux&authorized_key%5B%5D=SHA256%3Akey+one&keyboard=de",
    );
    let linux =
        RobotLinuxActivateRequest::new(number(), value("Debian 13 minimal"), value("en"), &keys)
            .unwrap_or_else(|_| unreachable!());
    assert_activation(
        &linux,
        "/boot/321/linux",
        "robot_activate_linux",
        OperationImpact::Destructive,
        b"dist=Debian+13+minimal&lang=en&authorized_key%5B%5D=SHA256%3Akey+one",
    );
    let vnc = RobotVncActivateRequest::new(number(), value("rescue-vnc"), value("en_US"));
    assert_activation(
        &vnc,
        "/boot/321/vnc",
        "robot_activate_vnc",
        OperationImpact::Destructive,
        b"dist=rescue-vnc&lang=en_US",
    );
    let windows = RobotWindowsActivateRequest::new(
        number(),
        value("Windows Server 2022 Standard Edition"),
        value("en"),
    );
    assert_activation(
        &windows,
        "/boot/321/windows",
        "robot_activate_windows",
        OperationImpact::Destructive,
        b"lang=en&os=Windows+Server+2022+Standard+Edition",
    );
}

#[test]
fn decodes_protected_active_state_and_exact_selection() {
    let request = RobotRescueActivateRequest::new(number(), value("linux"), &[], None)
        .unwrap_or_else(|_| unreachable!());
    let entry = decode_bound(&request, RESCUE_ACTIVE, |checked| checked.decode_response())
        .unwrap_or_else(|_| unreachable!("active Rescue response failed"));
    assert_eq!(entry.family(), RobotBootFamily::Rescue);
    assert!(entry.is_active());
    assert_eq!(entry.authorized_keys().len(), 1);
    assert_eq!(entry.host_keys().len(), 1);
    assert!(entry.password().is_some());
    assert!(!format!("{entry:?}").contains("generated-password"));
}

#[test]
fn deactivation_and_overview_require_exact_state_and_identity() {
    let request = RobotLinuxDeactivateRequest::new(number());
    let entry = decode_bound(&request, LINUX_INACTIVE, |checked| {
        checked.decode_response()
    })
    .unwrap_or_else(|_| unreachable!("inactive Linux response failed"));
    assert!(!entry.is_active());
    assert!(!entry.primary_choice().is_selected());

    let overview = format!(
        "{{\"boot\":{{\"rescue\":{},\"linux\":{},\"vnc\":{},\"windows\":{}}}}}",
        inner(RESCUE_ACTIVE, "rescue"),
        inner(LINUX_INACTIVE, "linux"),
        inner(VNC_INACTIVE, "vnc"),
        inner(WINDOWS_INACTIVE, "windows"),
    );
    let request = RobotBootGetRequest::new(number());
    let boot = decode_bound(&request, overview.as_bytes(), |checked| {
        checked.decode_response()
    })
    .unwrap_or_else(|_| unreachable!("boot overview failed"));
    assert_eq!(boot.rescue().server_number(), &number());
    assert_eq!(boot.windows().family(), RobotBootFamily::Windows);

    let mismatched = text(LINUX_INACTIVE).replace("321", "322");
    let request = RobotLinuxGetRequest::new(number());
    assert_eq!(
        decode_bound(&request, mismatched.as_bytes(), |checked| checked
            .decode_response())
        .err(),
        Some(RobotBootDecodeError::ResponseIdentityMismatch),
    );
}

#[test]
fn strict_decoder_rejects_unknown_fields_bad_families_and_state_mismatch() {
    let unknown =
        text(LINUX_INACTIVE).replace("\"password\":null", "\"password\":null,\"future\":true");
    let request = RobotLinuxGetRequest::new(number());
    assert_eq!(
        decode_bound(&request, unknown.as_bytes(), |checked| checked
            .decode_response())
        .err(),
        Some(RobotBootDecodeError::InvalidEnvelope),
    );
    let family = text(LINUX_INACTIVE).replace("2001:db8::", "192.0.2.11");
    let request = RobotLinuxGetRequest::new(number());
    assert_eq!(
        decode_bound(&request, family.as_bytes(), |checked| checked
            .decode_response())
        .err(),
        Some(RobotBootDecodeError::InvalidAddress),
    );
    let request = RobotLinuxDeactivateRequest::new(number());
    assert_eq!(
        decode_bound(&request, RESCUE_ACTIVE, |checked| checked.decode_response()).err(),
        Some(RobotBootDecodeError::InvalidEnvelope),
    );
}

#[test]
fn invalid_inputs_and_failed_preparation_fail_closed() {
    assert!(RobotBootValue::new("").is_err());
    assert!(RobotBootValue::new("line\nbreak").is_err());
    assert!(RobotBootValue::new("hidden\u{202e}text").is_err());
    let keys = vec![
        RobotBootKey::new("SHA256:key").unwrap_or_else(|_| unreachable!());
        MAX_ROBOT_BOOT_AUTHORIZED_KEYS + 1
    ];
    assert_eq!(
        RobotRescueActivateRequest::new(number(), value("linux"), &keys, None).err(),
        Some(RobotBootRequestError::TooManyAuthorizedKeys),
    );
    let duplicate = [
        RobotBootKey::new("SHA256:key").unwrap_or_else(|_| unreachable!()),
        RobotBootKey::new("SHA256:key").unwrap_or_else(|_| unreachable!()),
    ];
    assert_eq!(
        RobotLinuxActivateRequest::new(number(), value("Debian"), value("en"), &duplicate).err(),
        Some(RobotBootRequestError::DuplicateAuthorizedKey),
    );
    let request = RobotWindowsGetRequest::new(
        RobotServerNumber::new(u64::MAX).unwrap_or_else(|_| unreachable!()),
    );
    let mut target = [0xa5_u8; 4];
    let mut body = [0x5a_u8; 8];
    assert!(
        request
            .prepare(PreparationStorage::new(&mut target, &mut body))
            .is_err()
    );
    assert_eq!(target, [0; 4]);
    assert_eq!(body, [0; 8]);
}

#[test]
fn deprecated_windows_distribution_is_validated_before_discard() {
    let duplicate = text(WINDOWS_INACTIVE).replace(
        r#""dist":["standard"]"#,
        r#""dist":["standard","standard"]"#,
    );
    let request = RobotWindowsGetRequest::new(number());
    assert_eq!(
        decode_bound(&request, duplicate.as_bytes(), |checked| checked
            .decode_response())
        .err(),
        Some(RobotBootDecodeError::InvalidCollection),
    );
}

fn assert_activation<R: PrepareOperation<Error = RobotBootRequestError>>(
    request: &R,
    path: &str,
    id: &str,
    impact: OperationImpact,
    expected_body: &[u8],
) {
    let mut target = [0_u8; 96];
    let mut body = [0_u8; 512];
    let prepared = request
        .prepare(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("activation preparation failed"));
    assert_eq!(prepared.transport_request().body(), expected_body);
    assert_prepared(prepared, Method::Post, path, id, impact, true);
}

fn assert_prepared(
    prepared: cloud_sdk::operation::PreparedRequest<'_>,
    method: Method,
    path: &str,
    id: &str,
    impact: OperationImpact,
    has_body: bool,
) {
    assert_eq!(prepared.transport_request().method(), method);
    assert_eq!(prepared.transport_request().target().as_str(), path);
    assert_eq!(
        prepared.operation_id().map(|value| value.as_str()),
        Some(id)
    );
    assert_eq!(prepared.metadata().impact(), impact);
    assert_eq!(
        prepared.metadata().semantics(),
        if impact == OperationImpact::ReadOnly {
            RequestSemantics::Safe
        } else {
            RequestSemantics::NonIdempotent
        }
    );
    assert_eq!(
        prepared.metadata().retry_eligibility(),
        if impact == OperationImpact::ReadOnly {
            RetryEligibility::ExplicitPolicy
        } else {
            RetryEligibility::Never
        }
    );
    assert_eq!(
        prepared.body_sensitivity(),
        if has_body {
            RequestBodySensitivity::Sensitive
        } else {
            RequestBodySensitivity::Public
        }
    );
    assert_eq!(
        prepared.response_policy().max_body_bytes(),
        MAX_ROBOT_BOOT_RESPONSE_BYTES
    );
}

fn decode_bound<R, O>(
    request: &R,
    body: &[u8],
    decode: impl for<'a> FnOnce(CheckedRobotBoot<'a, '_, R>) -> O,
) -> O
where
    R: PrepareOperation<Error = RobotBootRequestError>,
{
    let mut target = [0_u8; 96];
    let mut request_body = [0_u8; 512];
    let inner = request
        .prepare(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("decode preparation failed"));
    let prepared = PreparedRobotBoot { request, inner };
    let mut storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut storage, body.len(), &mut headers);
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
    drop(attempt);
    decode(
        prepared
            .validate_response(response)
            .unwrap_or_else(|_| unreachable!()),
    )
}

fn inner<'a>(body: &'a [u8], field: &str) -> &'a str {
    let text = text(body);
    text.strip_prefix(&format!("{{\"{field}\":"))
        .and_then(|value| value.strip_suffix('}'))
        .unwrap_or_else(|| unreachable!())
}

fn number() -> RobotServerNumber {
    RobotServerNumber::new(321).unwrap_or_else(|_| unreachable!())
}
fn value(value: &str) -> RobotBootValue<'_> {
    RobotBootValue::new(value).unwrap_or_else(|_| unreachable!())
}
fn keyboard(value: &str) -> RobotKeyboardLayout<'_> {
    RobotKeyboardLayout::new(value).unwrap_or_else(|_| unreachable!())
}
fn text(value: &[u8]) -> &str {
    core::str::from_utf8(value).unwrap_or_else(|_| unreachable!())
}
