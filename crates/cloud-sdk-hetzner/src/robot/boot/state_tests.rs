use alloc::{format, string::String, vec};

use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::*;
use crate::robot::RobotServerNumber;

#[test]
fn official_overview_shape_accepts_inactive_windows_null_language() {
    let body = overview([false; 4]);
    let decoded = decode_bound(
        &RobotBootGetRequest::new(number()),
        body.as_bytes(),
        |checked| checked.decode_response(),
    )
    .unwrap_or_else(|_| unreachable!("official overview shape was rejected"));

    assert!(decoded.windows().languages().is_none());
    assert!(!decoded.windows().is_active());
}

#[test]
fn overview_accepts_zero_or_one_active_family_and_rejects_every_pair() {
    assert!(decode_overview([false; 4]).is_ok());
    for active in 0..4 {
        let mut states = [false; 4];
        *states
            .get_mut(active)
            .unwrap_or_else(|| unreachable!("active family index exceeded fixture")) = true;
        assert!(decode_overview(states).is_ok());
    }
    for (left, right) in [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)] {
        let mut states = [false; 4];
        *states
            .get_mut(left)
            .unwrap_or_else(|| unreachable!("left family index exceeded fixture")) = true;
        *states
            .get_mut(right)
            .unwrap_or_else(|| unreachable!("right family index exceeded fixture")) = true;
        assert_eq!(
            decode_overview(states).err(),
            Some(RobotBootDecodeError::MutationOutcomeMismatch),
        );
    }
}

#[test]
fn current_state_requires_password_and_selected_configuration_to_match_active() {
    let request = RobotLinuxGetRequest::new(number());
    for body in [
        linux_entry(true, false, true, true),
        linux_entry(true, true, false, true),
        linux_entry(true, true, true, false),
        linux_entry(false, true, false, false),
        linux_entry(false, false, true, false),
        linux_entry(false, false, false, true),
    ] {
        assert_eq!(
            decode_bound(&request, body.as_bytes(), |checked| checked
                .decode_response())
            .err(),
            Some(RobotBootDecodeError::MutationOutcomeMismatch),
        );
    }
}

#[test]
fn last_state_retains_exact_selection_but_tracks_current_password_state() {
    let request = RobotLinuxLastRequest::new(number());
    let inactive_selected = linux_entry(false, false, true, true);
    let decoded = decode_bound(&request, inactive_selected.as_bytes(), |checked| {
        checked.decode_response()
    })
    .unwrap_or_else(|_| unreachable!("inactive last selection was rejected"));
    assert!(!decoded.is_active());
    assert!(decoded.primary_choice().is_selected());

    let available = linux_entry(false, false, false, false);
    assert_eq!(
        decode_bound(&request, available.as_bytes(), |checked| checked
            .decode_response())
        .err(),
        Some(RobotBootDecodeError::MutationOutcomeMismatch),
    );
}

#[test]
fn mutation_shapes_reject_state_incoherence_before_request_value_matching() {
    let activation = RobotLinuxActivateRequest::new(
        number(),
        RobotBootValue::new("Debian 13").unwrap_or_else(|_| unreachable!()),
        RobotBootValue::new("en").unwrap_or_else(|_| unreachable!()),
        &[],
    )
    .unwrap_or_else(|_| unreachable!());
    let available = linux_entry(true, true, false, false);
    assert_eq!(
        decode_bound(&activation, available.as_bytes(), |checked| checked
            .decode_response())
        .err(),
        Some(RobotBootDecodeError::MutationOutcomeMismatch),
    );

    let deactivation = RobotLinuxDeactivateRequest::new(number());
    let selected = linux_entry(false, false, true, true);
    assert_eq!(
        decode_bound(&deactivation, selected.as_bytes(), |checked| checked
            .decode_response())
        .err(),
        Some(RobotBootDecodeError::MutationOutcomeMismatch),
    );
}

#[test]
fn null_windows_language_is_narrowly_limited_to_inactive_overview() {
    let body = windows_entry(false, false, false, true);
    let request = RobotWindowsGetRequest::new(number());
    assert_eq!(
        decode_bound(&request, body.as_bytes(), |checked| checked
            .decode_response())
        .err(),
        Some(RobotBootDecodeError::InvalidEnvelope),
    );

    let active_null = overview_entry("windows", true, true);
    let body = format!(
        "{{\"boot\":{{\"rescue\":{},\"linux\":{},\"vnc\":{},\"windows\":{active_null}}}}}",
        overview_entry("rescue", false, false),
        overview_entry("linux", false, false),
        overview_entry("vnc", false, false),
    );
    assert_eq!(
        decode_bound(
            &RobotBootGetRequest::new(number()),
            body.as_bytes(),
            |checked| { checked.decode_response() }
        )
        .err(),
        Some(RobotBootDecodeError::InvalidEnvelope),
    );
}

fn decode_overview(states: [bool; 4]) -> Result<RobotBoot, RobotBootDecodeError> {
    let body = overview(states);
    decode_bound(
        &RobotBootGetRequest::new(number()),
        body.as_bytes(),
        |checked| checked.decode_response(),
    )
}

fn overview(states: [bool; 4]) -> String {
    let [rescue, linux, vnc, windows] = states;
    format!(
        "{{\"boot\":{{\"rescue\":{},\"linux\":{},\"vnc\":{},\"windows\":{}}}}}",
        overview_entry("rescue", rescue, false),
        overview_entry("linux", linux, false),
        overview_entry("vnc", vnc, false),
        overview_entry("windows", windows, !windows),
    )
}

fn overview_entry(family: &str, active: bool, null_language: bool) -> String {
    let password = if active { "\"generated\"" } else { "null" };
    match family {
        "rescue" => format!(
            "{{\"server_ip\":\"192.0.2.10\",\"server_ipv6_net\":\"2001:db8::\",\"server_number\":321,\"os\":{},\"active\":{active},\"password\":{password},\"authorized_key\":[],\"host_key\":[]}}",
            choice(active, "\"linux\"", "[\"linux\"]"),
        ),
        "linux" => linux_object(active, active, active, active),
        "vnc" => format!(
            "{{\"server_ip\":\"192.0.2.10\",\"server_ipv6_net\":\"2001:db8::\",\"server_number\":321,\"dist\":{},\"lang\":{},\"active\":{active},\"password\":{password}}}",
            choice(active, "\"rescue-vnc\"", "[\"rescue-vnc\"]"),
            choice(active, "\"en_US\"", "[\"en_US\"]"),
        ),
        "windows" => windows_object(active, active, active, null_language),
        _ => unreachable!("test family is source locked"),
    }
}

const fn choice(active: bool, selected: &'static str, available: &'static str) -> &'static str {
    if active { selected } else { available }
}

fn linux_entry(active: bool, password: bool, primary: bool, language: bool) -> String {
    format!(
        "{{\"linux\":{}}}",
        linux_object(active, password, primary, language)
    )
}

fn linux_object(active: bool, password: bool, primary: bool, language: bool) -> String {
    format!(
        "{{\"server_ip\":\"192.0.2.10\",\"server_ipv6_net\":\"2001:db8::\",\"server_number\":321,\"dist\":{},\"lang\":{},\"active\":{active},\"password\":{},\"authorized_key\":[],\"host_key\":[]}}",
        if primary {
            "\"Debian 13\""
        } else {
            "[\"Debian 13\"]"
        },
        if language { "\"en\"" } else { "[\"en\"]" },
        if password { "\"generated\"" } else { "null" },
    )
}

fn windows_entry(active: bool, password: bool, primary: bool, null_language: bool) -> String {
    format!(
        "{{\"windows\":{}}}",
        windows_object(active, password, primary, null_language)
    )
}

fn windows_object(active: bool, password: bool, primary: bool, null_language: bool) -> String {
    let language = if null_language {
        "null"
    } else if active {
        "\"en\""
    } else {
        "[\"en\"]"
    };
    format!(
        "{{\"server_ip\":\"192.0.2.10\",\"server_ipv6_net\":\"2001:db8::\",\"server_number\":321,\"dist\":null,\"os\":{},\"lang\":{language},\"active\":{active},\"password\":{}}}",
        if primary {
            "\"Windows Server 2022\""
        } else {
            "[\"Windows Server 2022\"]"
        },
        if password { "\"generated\"" } else { "null" },
    )
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
        .unwrap_or_else(|_| unreachable!("test request preparation failed"));
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

fn number() -> RobotServerNumber {
    RobotServerNumber::new(321).unwrap_or_else(|_| unreachable!())
}
