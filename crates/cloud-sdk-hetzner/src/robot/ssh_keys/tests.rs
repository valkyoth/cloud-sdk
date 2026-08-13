use alloc::{format, vec};

use cloud_sdk::Method;
use cloud_sdk::operation::{
    CostIntent, OperationImpact, OperationMetadata, PreparationStorage, PrepareOperation,
    RequestBodySensitivity, RequestIdPolicy, RequestSemantics, RetryEligibility,
};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::*;
use crate::security::shared::SshAlgorithm;

pub(super) const PUBLIC_KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILM+rvN+ot98qgEN796jTiQfZfG1KaT0PtFDJ/XFSqti user@example.com";
const SSH2_KEY: &str = "---- BEGIN SSH2 PUBLIC KEY ----\nComment: deploy\nAAAAC3NzaC1lZDI1NTE5AAAAILM+rvN+ot98qgEN796jTiQfZfG1KaT0PtFDJ/XFSqti\n---- END SSH2 PUBLIC KEY ----";
pub(super) const FINGERPRINT: &str = "ae:6f:ba:1b:70:2c:ae:c7:5c:ab:6e:4d:5e:d4:c7:23";
pub(super) const ENTRY: &[u8] = br#"{"key":{"name":"deploy-key","fingerprint":"ae:6f:ba:1b:70:2c:ae:c7:5c:ab:6e:4d:5e:d4:c7:23","type":"ED25519","size":256,"data":"ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILM+rvN+ot98qgEN796jTiQfZfG1KaT0PtFDJ/XFSqti user@example.com","created_at":"2021-12-31 23:59:59"}}"#;

#[test]
fn prepares_all_five_source_locked_operations() {
    assert_prepared(
        RobotSshKeyListRequest::new(),
        Method::Get,
        "/key",
        "robot_list_ssh_keys",
        OperationImpact::ReadOnly,
        RequestBodySensitivity::Public,
        MAX_ROBOT_SSH_KEY_LIST_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotSshKeyCreateRequest::new(name("deploy-key"), data()),
        Method::Post,
        "/key",
        "robot_create_ssh_key",
        OperationImpact::Mutation,
        RequestBodySensitivity::Sensitive,
        MAX_ROBOT_SSH_KEY_ITEM_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotSshKeyGetRequest::new(fingerprint()),
        Method::Get,
        &format!("/key/{FINGERPRINT}"),
        "robot_get_ssh_key",
        OperationImpact::ReadOnly,
        RequestBodySensitivity::Public,
        MAX_ROBOT_SSH_KEY_ITEM_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotSshKeyUpdateRequest::new(fingerprint(), name("renamed")),
        Method::Post,
        &format!("/key/{FINGERPRINT}"),
        "robot_update_ssh_key",
        OperationImpact::Mutation,
        RequestBodySensitivity::Sensitive,
        MAX_ROBOT_SSH_KEY_ITEM_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotSshKeyDeleteRequest::new(fingerprint()),
        Method::Delete,
        &format!("/key/{FINGERPRINT}"),
        "robot_delete_ssh_key",
        OperationImpact::Destructive,
        RequestBodySensitivity::Public,
        0,
    );
}

#[test]
fn forms_are_atomic_encoded_and_sensitive() {
    let create = RobotSshKeyCreateRequest::new(name("deploy key"), data());
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 1_024];
    let prepared = create
        .prepare(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("create fixture failed"));
    let text = core::str::from_utf8(prepared.transport_request().body())
        .unwrap_or_else(|_| unreachable!("form lost UTF-8"));
    assert!(text.starts_with("name=deploy+key&data=ssh-ed25519+"));
    assert!(text.contains("%2B"));
    assert!(text.contains("%2F"));
    assert!(!text.contains("user@example.com\n"));

    let update = RobotSshKeyUpdateRequest::new(fingerprint(), name("new name"));
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 128];
    let prepared = update
        .prepare(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("update fixture failed"));
    assert_eq!(prepared.transport_request().body(), b"name=new+name");
}

#[test]
fn checked_models_validate_crypto_metadata_and_redact() {
    let key = decode_get(ENTRY).unwrap_or_else(|_| unreachable!("valid key fixture failed"));
    assert_eq!(key.algorithm(), RobotSshKeyAlgorithm::Ed25519);
    assert_eq!(key.size_bits(), 256);
    assert_ne!(key.sha256_fingerprint(), &[0_u8; 32]);
    assert_eq!(
        key.created_at()
            .with_components(|year, month, day, hour, minute, second| {
                (year, month, day, hour, minute, second)
            }),
        (2021, 12, 31, 23, 59, 59)
    );
    let debug = format!("{key:?}");
    assert!(!debug.contains("deploy-key"));
    assert!(!debug.contains(FINGERPRINT));
    assert!(!debug.contains("AAAAC3"));
}

#[test]
fn every_admitted_wire_algorithm_has_an_exact_source_family() {
    use super::decode::source_algorithm;

    for (wire, source) in [
        (SshAlgorithm::Rsa, RobotSshKeyAlgorithm::Rsa),
        (SshAlgorithm::Ed25519, RobotSshKeyAlgorithm::Ed25519),
        (SshAlgorithm::SkEd25519, RobotSshKeyAlgorithm::Ed25519),
        (SshAlgorithm::EcdsaNistP256, RobotSshKeyAlgorithm::Ecdsa),
        (SshAlgorithm::EcdsaNistP384, RobotSshKeyAlgorithm::Ecdsa),
        (SshAlgorithm::EcdsaNistP521, RobotSshKeyAlgorithm::Ecdsa),
        (SshAlgorithm::SkEcdsaNistP256, RobotSshKeyAlgorithm::Ecdsa),
    ] {
        assert_eq!(source_algorithm(wire), source);
    }
}

#[test]
fn strict_models_reject_cross_field_mismatches() {
    for invalid in [
        text(ENTRY).replace("\"size\":256", "\"size\":255"),
        text(ENTRY).replace("\"ED25519\"", "\"RSA\""),
        text(ENTRY).replace("ae:6f", "00:6f"),
        text(ENTRY).replace("2021-12-31", "2021-02-29"),
        text(ENTRY).replace("23:59:59", "24:00:00"),
        text(ENTRY).replace("\"created_at\"", "\"future\":1,\"created_at\""),
    ] {
        assert!(decode_get(invalid.as_bytes()).is_err());
    }
}

#[test]
fn list_rejects_duplicate_fingerprints_and_extra_fields() {
    let entry = text(ENTRY);
    let list = format!("[{entry}]");
    let decoded = decode_list(list.as_bytes())
        .unwrap_or_else(|_| unreachable!("valid key list fixture failed"));
    assert_eq!(decoded.len(), 1);
    assert!(!decoded.is_empty());
    assert_eq!(
        decode_list(format!("[{entry},{entry}]").as_bytes()).err(),
        Some(RobotSshKeyDecodeError::InvalidList)
    );
}

#[test]
fn response_association_rejects_wrong_identity_and_mutation_outcome() {
    let wrong_get = RobotSshKeyGetRequest::new(
        RobotSshKeyFingerprint::new("00:11:22:33:44:55:66:77:88:99:aa:bb:cc:dd:ee:ff")
            .unwrap_or_else(|_| unreachable!("alternate fingerprint failed")),
    );
    assert_eq!(
        decode_bound_get(&wrong_get, ENTRY).err(),
        Some(RobotSshKeyDecodeError::ResponseIdentityMismatch)
    );

    let create = RobotSshKeyCreateRequest::new(name("other"), data());
    assert_eq!(
        decode_create(&create, ENTRY).err(),
        Some(RobotSshKeyDecodeError::MutationOutcomeMismatch)
    );
    let update = RobotSshKeyUpdateRequest::new(fingerprint(), name("other"));
    assert_eq!(
        decode_update(&update, ENTRY).err(),
        Some(RobotSshKeyDecodeError::MutationOutcomeMismatch)
    );
}

#[test]
fn rfc4716_create_normalizes_to_the_response_key_identity() {
    let request = RobotSshKeyCreateRequest::new(
        name("deploy-key"),
        RobotSshKeyData::new(SSH2_KEY)
            .unwrap_or_else(|_| unreachable!("valid RFC 4716 fixture failed")),
    );
    assert!(decode_create(&request, ENTRY).is_ok());
}

#[test]
fn delete_requires_exact_empty_ok_response() {
    let request = RobotSshKeyDeleteRequest::new(fingerprint());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("delete preparation failed"));
    with_response(StatusCode::OK, b"", None, |response| {
        assert!(
            prepared
                .validate_response(response)
                .unwrap_or_else(|_| unreachable!("empty delete response failed"))
                .decode_response()
                .is_ok()
        );
    });
}

#[test]
fn failed_preparation_clears_all_caller_storage() {
    let request = RobotSshKeyCreateRequest::new(name("deploy-key"), data());
    let mut target = [0xa5_u8; 2];
    let mut body = [0x5a_u8; 4];
    assert!(
        request
            .prepare(PreparationStorage::new(&mut target, &mut body))
            .is_err()
    );
    assert_eq!(target, [0_u8; 2]);
    assert_eq!(body, [0_u8; 4]);
}

#[test]
fn prepared_policy_failure_precedes_borrow_and_clears_sensitive_storage() {
    let request = RobotSshKeyCreateRequest::new(name("deploy-key"), data());
    let metadata = OperationMetadata::new(
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Discard,
    )
    .unwrap_or_else(|_| unreachable!("late-failure metadata fixture failed"));
    let mut target = [0xa5_u8; 128];
    let mut body = [0x5a_u8; 1_024];
    let result = super::prepare::prepare_with_metadata(
        super::prepare::Kind::Create(&request.name, &request.data),
        PreparationStorage::new(&mut target, &mut body),
        Ok(metadata),
    );
    assert!(matches!(
        result,
        Err(RobotSshKeyRequestError::InvalidPreparedPolicy(
            cloud_sdk::operation::PreparedRequestPolicyError::ReadOnlyMethodMismatch
        ))
    ));
    assert_eq!(target, [0_u8; 128]);
    assert_eq!(body, [0_u8; 1_024]);
}

#[allow(clippy::too_many_arguments)]
fn assert_prepared<O>(
    operation: O,
    method: Method,
    target: &str,
    operation_id: &str,
    impact: OperationImpact,
    sensitivity: RequestBodySensitivity,
    maximum: usize,
) where
    O: PrepareOperation<Error = RobotSshKeyRequestError>,
{
    let mut target_storage = [0_u8; 128];
    let mut body_storage = [0_u8; 16_384];
    let prepared = operation
        .prepare(PreparationStorage::new(
            &mut target_storage,
            &mut body_storage,
        ))
        .unwrap_or_else(|_| unreachable!("SSH-key preparation failed"));
    assert_eq!(prepared.transport_request().method(), method);
    assert_eq!(prepared.transport_request().target().as_str(), target);
    assert_eq!(
        prepared.operation_id().map(|value| value.as_str()),
        Some(operation_id)
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
    assert_eq!(prepared.body_sensitivity(), sensitivity);
    assert_eq!(prepared.response_policy().max_body_bytes(), maximum);
}

fn decode_list(body: &[u8]) -> Result<RobotSshKeyList, RobotSshKeyDecodeError> {
    let request = RobotSshKeyListRequest::new();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("list preparation failed"));
    with_json(prepared, StatusCode::OK, body, |checked| {
        checked.decode_response()
    })
}

fn decode_get(body: &[u8]) -> Result<RobotSshKey, RobotSshKeyDecodeError> {
    let request = RobotSshKeyGetRequest::new(fingerprint());
    decode_bound_get(&request, body)
}

fn decode_bound_get(
    request: &RobotSshKeyGetRequest,
    body: &[u8],
) -> Result<RobotSshKey, RobotSshKeyDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("get preparation failed"));
    with_json(prepared, StatusCode::OK, body, |checked| {
        checked.decode_response()
    })
}

fn decode_create(
    request: &RobotSshKeyCreateRequest<'_>,
    body: &[u8],
) -> Result<RobotSshKey, RobotSshKeyDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 16_384];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("create preparation failed"));
    with_json(prepared, StatusCode::CREATED, body, |checked| {
        checked.decode_response()
    })
}

fn decode_update(
    request: &RobotSshKeyUpdateRequest,
    body: &[u8],
) -> Result<RobotSshKey, RobotSshKeyDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 256];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("update preparation failed"));
    with_json(prepared, StatusCode::OK, body, |checked| {
        checked.decode_response()
    })
}

fn with_json<'request, R, O>(
    prepared: PreparedRobotSshKey<'_, 'request, R>,
    status: StatusCode,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotSshKey<'_, 'request, R>) -> O,
) -> O {
    let mut result = None;
    with_response(status, body, Some("application/json"), |response| {
        let checked = prepared
            .validate_response(response)
            .unwrap_or_else(|_| unreachable!("response policy failed"));
        result = Some(decode(checked));
    });
    result.unwrap_or_else(|| unreachable!("response was not decoded"))
}

fn with_response<R>(
    status: StatusCode,
    body: &[u8],
    content_type: Option<&str>,
    inspect: impl FnOnce(ResponseBuffer<'_>) -> R,
) -> R {
    let mut storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut storage, body.len(), &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("response attempt failed"));
    if let Some(content_type) = content_type {
        attempt
            .headers_mut()
            .unwrap_or_else(|_| unreachable!("response headers failed"))
            .try_push(
                "content-type",
                content_type.as_bytes(),
                HeaderSensitivity::Public,
            )
            .unwrap_or_else(|_| unreachable!("content type failed"));
    }
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("response body failed"))
        .copy_from_slice(body);
    attempt
        .commit(status, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("response commit failed"));
    drop(attempt);
    inspect(response)
}

pub(super) fn name(value: &str) -> RobotSshKeyName {
    RobotSshKeyName::new(value).unwrap_or_else(|_| unreachable!("name fixture failed"))
}

pub(super) fn fingerprint() -> RobotSshKeyFingerprint {
    RobotSshKeyFingerprint::new(FINGERPRINT)
        .unwrap_or_else(|_| unreachable!("fingerprint fixture failed"))
}

pub(super) fn data() -> RobotSshKeyData<'static> {
    RobotSshKeyData::new(PUBLIC_KEY).unwrap_or_else(|_| unreachable!("key fixture failed"))
}

fn text(value: &[u8]) -> &str {
    core::str::from_utf8(value).unwrap_or_else(|_| unreachable!("fixture lost UTF-8"))
}
