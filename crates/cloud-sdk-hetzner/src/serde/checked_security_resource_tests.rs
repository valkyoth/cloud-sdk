//! Dedicated certificate and SSH-key response regressions.

use alloc::format;

use cloud_sdk::transport::StatusCode;

use super::checked_fixtures::{action_value, resource_value};
use super::checked_test_support::{assert_decode_error, decode_response, prepared, response};
use super::checked_tests::pagination;
use super::{
    CertificateIssuanceState, CertificateRenewalState, HetznerDecodeError, HetznerSuccess,
    ResponseModelError, SecurityResource,
};
use crate::SECURITY_SERVICE_ID;
use crate::response::ApiErrorCode;

const VALID_SSH_KEY: &str = concat!(
    "ssh-ed25519 ",
    "AAAAC3NzaC1lZDI1NTE5AAAAILM+rvN+ot98qgEN796jTiQfZfG1KaT0PtFDJ/XFSqti ",
    "user@example.com"
);

fn body(root: &str, value: serde_json::Value) -> alloc::vec::Vec<u8> {
    serde_json::to_vec(&serde_json::json!({root:value})).unwrap_or_default()
}

fn assert_model_error(
    operation: &'static str,
    root: &str,
    value: serde_json::Value,
    expected: ResponseModelError,
) {
    let body = body(root, value);
    assert_decode_error(
        decode_response(
            prepared(operation, SECURITY_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, &body),
        ),
        HetznerDecodeError::Model(expected),
    );
}

#[test]
fn ssh_key_singletons_pages_and_rotation_responses_are_source_complete() {
    for operation in ["create_ssh_key", "get_ssh_key", "update_ssh_key"] {
        let status = if operation == "create_ssh_key" {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        };
        let body = body("ssh_key", resource_value("ssh_key"));
        let decoded = decode_response(
            prepared(operation, SECURITY_SERVICE_ID, status),
            response(status, &body),
        );
        let Ok(decoded) = decoded else {
            unreachable!("source-derived SSH-key fixture failed")
        };
        let HetznerSuccess::SecurityResource(SecurityResource::SshKey(key)) = decoded.success()
        else {
            unreachable!("SSH-key operation selected the wrong model")
        };
        assert_eq!(key.id(), 1);
        assert_eq!(key.name(), "x");
        assert_eq!(
            key.try_with_public_key(|value| value == VALID_SSH_KEY),
            Ok(true)
        );
        assert_eq!(
            key.sha256_fingerprint(),
            &[
                0x50, 0x25, 0x22, 0x2e, 0xbe, 0xcf, 0x8e, 0xcf, 0x70, 0x14, 0x52, 0x4c, 0x0c, 0x1c,
                0x8b, 0x81, 0xcd, 0xcd, 0xae, 0xd7, 0x54, 0xdf, 0x8e, 0x0e, 0x81, 0x43, 0x38, 0xe7,
                0x06, 0x4f, 0x70, 0x84,
            ]
        );
        let debug = format!("{key:?}");
        assert!(!debug.contains("Y2xvdWQ"));
        assert!(!debug.contains(key.fingerprint()));
    }

    let meta = serde_json::from_str::<serde_json::Value>(pagination()).unwrap_or_default();
    let page = serde_json::to_vec(&serde_json::json!({
        "ssh_keys":[resource_value("ssh_key")], "meta":meta
    }))
    .unwrap_or_default();
    let decoded = decode_response(
        prepared("list_ssh_keys", SECURITY_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &page),
    );
    let Ok(decoded) = decoded else {
        unreachable!("SSH-key page failed")
    };
    let HetznerSuccess::SecurityResources {
        resources,
        pagination: Some(pagination),
    } = decoded.success()
    else {
        unreachable!("SSH-key page selected the wrong model")
    };
    assert_eq!(resources.len(), 1);
    assert_eq!(pagination.total_entries(), Some(1));
}

#[test]
fn certificate_singletons_pages_and_create_composites_use_one_typed_family() {
    for operation in ["get_certificate", "update_certificate"] {
        let body = body("certificate", resource_value("certificate"));
        let decoded = decode_response(
            prepared(operation, SECURITY_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, &body),
        );
        assert!(matches!(
            decoded.map(|value| value.into_success()),
            Ok(HetznerSuccess::SecurityResource(
                SecurityResource::Certificate(_)
            ))
        ));
    }

    let meta = serde_json::from_str::<serde_json::Value>(pagination()).unwrap_or_default();
    let page = serde_json::to_vec(&serde_json::json!({
        "certificates":[resource_value("certificate")], "meta":meta
    }))
    .unwrap_or_default();
    let decoded = decode_response(
        prepared("list_certificates", SECURITY_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &page),
    );
    assert!(matches!(
        decoded.map(|value| value.into_success()),
        Ok(HetznerSuccess::SecurityResources { .. })
    ));

    let created = serde_json::to_vec(&serde_json::json!({
        "certificate":resource_value("certificate"), "action":action_value()
    }))
    .unwrap_or_default();
    let decoded = decode_response(
        prepared(
            "create_certificate",
            SECURITY_SERVICE_ID,
            StatusCode::CREATED,
        ),
        response(StatusCode::CREATED, &created),
    );
    let Ok(decoded) = decoded else {
        unreachable!("certificate create composite failed")
    };
    let HetznerSuccess::Composite(composite) = decoded.success() else {
        unreachable!("certificate create selected the wrong model")
    };
    assert!(matches!(
        composite.security_resource(),
        Some(SecurityResource::Certificate(_))
    ));
    assert!(composite.action().is_some());
    assert!(composite.resource().is_none());
}

#[test]
fn certificate_status_and_type_contradictions_fail_closed() {
    let mut unknown = resource_value("certificate");
    let Some(fields) = unknown.as_object_mut() else {
        unreachable!("certificate fixture is not an object")
    };
    fields.insert("type".into(), serde_json::json!("managed"));
    fields.insert(
        "status".into(),
        serde_json::json!({"issuance":"future","renewal":"scheduled","error":null}),
    );
    assert_model_error(
        "get_certificate",
        "certificate",
        unknown,
        ResponseModelError::UnknownEnumValue,
    );

    for status in [
        serde_json::json!({"issuance":"failed","renewal":"scheduled","error":null}),
        serde_json::json!({
            "issuance":"completed", "renewal":"scheduled",
            "error":{"code":"issuance_failed","message":"private diagnostic"}
        }),
    ] {
        let mut certificate = resource_value("certificate");
        let Some(fields) = certificate.as_object_mut() else {
            unreachable!("certificate fixture is not an object")
        };
        fields.insert("type".into(), serde_json::json!("managed"));
        fields.insert("status".into(), status);
        assert_model_error(
            "get_certificate",
            "certificate",
            certificate,
            ResponseModelError::EnvelopeMismatch,
        );
    }

    let mut valid_failure = resource_value("certificate");
    let Some(fields) = valid_failure.as_object_mut() else {
        unreachable!("certificate fixture is not an object")
    };
    fields.insert("type".into(), serde_json::json!("managed"));
    fields.insert(
        "status".into(),
        serde_json::json!({
            "issuance":"failed", "renewal":"scheduled",
            "error":{"code":"issuance_failed","message":"private diagnostic"}
        }),
    );
    let body = body("certificate", valid_failure);
    let decoded = decode_response(
        prepared("get_certificate", SECURITY_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &body),
    );
    let Ok(decoded) = decoded else {
        unreachable!("coherent failed certificate was rejected")
    };
    let HetznerSuccess::SecurityResource(SecurityResource::Certificate(certificate)) =
        decoded.success()
    else {
        unreachable!("certificate failure selected the wrong model")
    };
    let Some(status) = certificate.status() else {
        unreachable!("certificate status disappeared")
    };
    assert_eq!(status.issuance(), Some(CertificateIssuanceState::Failed));
    assert_eq!(status.renewal(), Some(CertificateRenewalState::Scheduled));
    let Some(error) = status.error() else {
        unreachable!("certificate error disappeared")
    };
    assert_eq!(error.code(), ApiErrorCode::Unknown);
    assert_eq!(error.code_text(), "issuance_failed");
    assert!(!format!("{error:?}").contains("issuance_failed"));
    assert!(!format!("{certificate:?}").contains("private diagnostic"));
}

#[test]
fn ssh_key_shape_and_certificate_chain_bounds_fail_closed() {
    for (field, value) in [
        ("fingerprint", serde_json::json!("00:11")),
        ("public_key", serde_json::json!("ssh-dss Y2xvdWQ=")),
        (
            "public_key",
            serde_json::json!("ssh-ed25519 Y2xvdWQtc2RrLXRlc3Q="),
        ),
        (
            "public_key",
            serde_json::json!("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAA==="),
        ),
        (
            "public_key",
            serde_json::json!("ecdsa-sha2-attacker@example.com AAAA"),
        ),
        (
            "public_key",
            serde_json::json!("sk-ssh-ed25519@evil.example AAAA"),
        ),
        ("created", serde_json::json!("2026-02-30T00:00:00Z")),
    ] {
        let mut key = resource_value("ssh_key");
        let Some(fields) = key.as_object_mut() else {
            unreachable!("SSH-key fixture is not an object")
        };
        fields.insert(field.into(), value);
        assert_model_error(
            "get_ssh_key",
            "ssh_key",
            key,
            ResponseModelError::InvalidText,
        );
    }

    let mut mismatched = resource_value("ssh_key");
    let Some(fields) = mismatched.as_object_mut() else {
        unreachable!("SSH-key fixture is not an object")
    };
    fields.insert(
        "fingerprint".into(),
        serde_json::json!("00:11:22:33:44:55:66:77:88:99:aa:bb:cc:dd:ee:ff"),
    );
    assert_model_error(
        "get_ssh_key",
        "ssh_key",
        mismatched,
        ResponseModelError::EnvelopeMismatch,
    );

    let block = "-----BEGIN CERTIFICATE-----\nYQ==\n-----END CERTIFICATE-----";
    let mut certificate = resource_value("certificate");
    let Some(fields) = certificate.as_object_mut() else {
        unreachable!("certificate fixture is not an object")
    };
    fields.insert(
        "certificate".into(),
        serde_json::json!(alloc::vec![block; 6].join("\n")),
    );
    assert_model_error(
        "get_certificate",
        "certificate",
        certificate,
        ResponseModelError::TooManyItems,
    );
}
