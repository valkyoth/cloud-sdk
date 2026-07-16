//! Security regressions for the checked response decoder.

use alloc::string::String;

use cloud_sdk::ApiFamily;
use cloud_sdk::transport::StatusCode;

use super::checked_tests::{prepared, response};
use super::strict_json::MAX_JSON_NODES;
use super::{HetznerDecodeError, decode_response};

#[test]
fn rejects_aggregate_json_amplification_and_protects_failure_path_strings() {
    let mut amplified = String::from(r#"{"server":{"id":1},"future":["#);
    let inner = "[null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null]";
    let containers = MAX_JSON_NODES / 17 + 1;
    for index in 0..containers {
        if index != 0 {
            amplified.push(',');
        }
        amplified.push_str(inner);
    }
    amplified.push_str("]}");

    assert!(containers < 4096);
    assert!(amplified.len() < 1_000_000);
    assert_eq!(
        decode_response(
            prepared("get_server", ApiFamily::Cloud, StatusCode::OK),
            response(StatusCode::OK, amplified.as_bytes()),
        ),
        Err(HetznerDecodeError::MalformedPayload)
    );

    for malformed in [
        br#"{"root_password":"first","root_password":"second"}"#.as_slice(),
        br#"{"root_password":"temporary"} trailing"#,
    ] {
        assert_eq!(
            decode_response(
                prepared("create_server", ApiFamily::Cloud, StatusCode::CREATED),
                response(StatusCode::CREATED, malformed),
            ),
            Err(HetznerDecodeError::MalformedPayload)
        );
    }
    assert!(matches!(
        decode_response(
            prepared("create_server", ApiFamily::Cloud, StatusCode::CREATED),
            response(StatusCode::CREATED, br#"{"root_password":"temporary"}"#),
        ),
        Err(HetznerDecodeError::Model(_))
    ));
}
