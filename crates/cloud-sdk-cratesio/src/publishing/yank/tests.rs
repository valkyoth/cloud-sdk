use super::*;
use crate::{
    credentials::{ApiToken, CredentialOrigin},
    discovery::tests::Fixture as _,
    identifiers::{CrateName, Version},
    wire::JsonSuccess,
};
use alloc::{format, vec};
use cloud_sdk::{
    rate_limit::WallClockTimestamp,
    transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode},
};
#[cfg(feature = "blocking")]
mod execution;

fn request(yanked: bool) -> YankRequest<'static> {
    let n = CrateName::new("serde").fixture("name");
    let v = Version::new("1.0.0").fixture("version");
    if yanked {
        YankRequest::yank(n, v)
    } else {
        YankRequest::unyank(n, v)
    }
}
fn token(origin: CredentialOrigin) -> ApiToken {
    use core::sync::atomic::{AtomicU8, Ordering};
    static NEXT: AtomicU8 = AtomicU8::new(0);
    let mut bytes = [b'a'.saturating_add(NEXT.fetch_add(1, Ordering::Relaxed) % 26); 32];
    ApiToken::from_mut_bytes(origin, &mut bytes).fixture("token")
}
fn admitted<R>(
    wire: &[u8],
    inspect: impl FnOnce(JsonSuccess<'_>) -> Result<R, YankError>,
) -> Result<R, YankError> {
    let mut bytes = vec![0xa5; wire.len()];
    let mut headers = [0xa5; 512];
    let mut response = ResponseBuffer::new(&mut bytes, wire.len(), &mut headers);
    let mut attempt = response.writer().begin_attempt().fixture("attempt");
    attempt.body_mut().fixture("body").copy_from_slice(wire);
    attempt
        .headers_mut()
        .fixture("headers")
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .fixture("type");
    attempt
        .commit(StatusCode::OK, wire.len(), ResponseMetadata::EMPTY)
        .fixture("commit");
    drop(attempt);
    let result = match crate::wire::JsonResponsePolicy::new(StatusCode::OK, wire.len())
        .fixture("policy")
        .admit(response, WallClockTimestamp::new(0))
    {
        Ok(success) => inspect(success),
        Err(_) => Err(YankError::Json),
    };
    assert!(bytes.iter().all(|b| *b == 0));
    assert!(headers.iter().all(|b| *b == 0));
    result
}
#[test]
fn exact_cargo_targets_and_mutation_classification() {
    for yanked in [true, false] {
        let r = request(yanked);
        let mut bytes = [0; 256];
        assert_eq!(
            r.write_target(&mut bytes).fixture("target").as_str(),
            if yanked {
                "/api/v1/crates/serde/1.0.0/yank"
            } else {
                "/api/v1/crates/serde/1.0.0/unyank"
            }
        );
        assert_eq!(
            r.operation().method(),
            if yanked {
                cloud_sdk::Method::Delete
            } else {
                cloud_sdk::Method::Put
            }
        );
        assert_eq!(
            r.operation().operation_name(),
            if yanked {
                "yank_version"
            } else {
                "unyank_version"
            }
        );
        assert_eq!(r.operation().requested_yanked(), yanked);
        assert!(r.operation().is_state_convergent());
        assert!(!r.operation().permits_automatic_retry());
        let mut short = [0xa5; 8];
        assert!(r.write_target(&mut short).is_err());
        assert_eq!(short, [0xa5; 8]);
        let token = token(CredentialOrigin::Production);
        assert_eq!(format!("{r:?}"), "YankRequest([redacted])");
        assert_eq!(format!("{:?}", r.confirm(&token)), "YankPermit([redacted])");
    }
    let r = YankRequest::yank(
        CrateName::new("crate_name").fixture("name"),
        Version::new("1.2.3-alpha.1+build.2").fixture("version"),
    );
    let mut bytes = [0; 256];
    assert_eq!(
        r.write_target(&mut bytes).fixture("target").as_str(),
        "/api/v1/crates/crate_name/1.2.3-alpha.1%2Bbuild.2/yank"
    );
    for bad in [
        "",
        "1",
        "1.0",
        "01.0.0",
        "1.0.0/unyank",
        "1.0.0?x",
        "1.0.0\n",
        "1.0.0-01",
        "1.0.0+",
    ] {
        assert!(Version::new(bad).is_err());
    }
}
#[test]
fn acknowledgement_is_not_an_observed_state_and_requires_true_ok() {
    let token = token(CredentialOrigin::Production);
    for yanked in [true, false] {
        for _ in 0..2 {
            let ack = admitted(br#"{"ok":true}"#, |s| {
                request(yanked).confirm(&token).decode_response(s)
            })
            .fixture("ack");
            assert_eq!(ack.requested_yanked(), yanked);
            let mut target = [0; 256];
            assert_eq!(
                ack.verification_request()
                    .write_target(&mut target)
                    .fixture("target")
                    .as_str(),
                "/api/v1/crates/serde/1.0.0"
            );
        }
        for wire in [
            br#"{}"#.as_slice(),
            br#"{"ok":false}"#,
            br#"{"ok":null}"#,
            br#"{"ok":1}"#,
            br#"{"ok":"true"}"#,
            br#"{"ok":true,"ok":false}"#,
            br#"{"ok":true,"ok":true}"#,
            br#"{"ok":true,"errors":[{"detail":"denied"}]}"#,
            br#"{"ok":true}{}"#,
            br#"[]"#,
        ] {
            assert!(
                admitted(wire, |s| request(yanked).confirm(&token).decode_response(s)).is_err()
            );
        }
    }
}
#[test]
fn read_back_binds_identity_and_exposes_stale_state_without_replay() {
    for origin in [CredentialOrigin::Production, CredentialOrigin::Staging] {
        let token = token(origin);
        for yanked in [true, false] {
            let ack = admitted(br#"{"ok":true}"#, |s| {
                request(yanked).confirm(&token).decode_response(s)
            })
            .fixture("ack");
            assert_eq!(
                ack.verification_endpoint(),
                match origin {
                    CredentialOrigin::Production =>
                        crate::endpoint::OfficialCratesIoEndpoint::production_api(),
                    CredentialOrigin::Staging =>
                        crate::endpoint::OfficialCratesIoEndpoint::staging_api(),
                }
            );
            for observed in [true, false] {
                let mut value: serde_json::Value =
                    serde_json::from_str(include_str!("../../versions/fixtures/find_version.json"))
                        .fixture("fixture");
                *value.pointer_mut("/version/yanked").fixture("state") =
                    serde_json::json!(observed);
                let wire = serde_json::to_vec(&value).fixture("wire");
                let result = admitted(&wire, |s| ack.decode_observed_state(s)).fixture("snapshot");
                assert_eq!(result.observed_yanked(), observed);
                assert_eq!(result.matches_requested_state(), yanked == observed);
                assert_eq!(
                    result
                        .record()
                        .fields()
                        .required("yanked")
                        .fixture("field")
                        .boolean()
                        .fixture("boolean"),
                    observed
                );
                for (field, wrong) in [("crate", "other"), ("num", "1.0.1"), ("num", "1.0.0+other")]
                {
                    let mut bad = value.clone();
                    *bad.pointer_mut(&format!("/version/{field}"))
                        .fixture("field") = serde_json::json!(wrong);
                    assert!(
                        admitted(&serde_json::to_vec(&bad).fixture("wire"), |s| ack
                            .decode_observed_state(s))
                        .is_err()
                    );
                }
            }
        }
    }
}
