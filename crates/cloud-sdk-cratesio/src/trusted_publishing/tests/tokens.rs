use super::*;
use crate::publishing::{PublishMetadata, PublishRequest};
pub(super) fn metadata(name: &str) -> alloc::vec::Vec<u8> {
    let metadata = json!({"name":name,"vers":"1.0.0","deps":[],"features":{},"authors":[],"description":"test","documentation":null,"homepage":null,"readme":null,"readme_file":null,"keywords":[],"categories":[],"license":"MIT","license_file":null,"repository":null,"badges":{},"links":null,"rust_version":null});
    serde_json::to_vec(&metadata).fixture("metadata")
}
pub(super) fn publish(metadata: &[u8]) -> PublishRequest<'_> {
    PublishRequest::new(
        PublishMetadata::from_json(metadata).fixture("parse"),
        1,
        cloud_sdk::transport::StreamLimits::new(1_000_000, 4096, 1_000_000, 4_000_000, 2)
            .fixture("limits"),
    )
    .fixture("publish")
}
#[test]
fn exchanged_token_is_private_locally_expiring_and_crate_bound() {
    let token = temporary(100);
    let intended = metadata("regex");
    let other = metadata("other");
    assert_eq!(token.local_deadline(), 1900);
    assert_eq!(format!("{token:?}"), "TemporaryToken([redacted])");
    assert!(token.confirm_publish(publish(&intended), 100).is_ok());
    assert!(token.confirm_publish(publish(&intended), 1899).is_ok());
    assert!(token.confirm_publish(publish(&intended), 1900).is_err());
    assert!(token.confirm_publish(publish(&intended), 99).is_err());
    assert!(token.confirm_publish(publish(&other), 100).is_err());
    let permit = TrustedPublishingPermit::confirm_revoke(token);
    assert_eq!(permit.operation(), TrustedPublishingOperation::Revoke);
    assert_eq!(permit.operation().method(), cloud_sdk::Method::Delete);
    let mut target = [0; 128];
    assert_eq!(
        permit.write_target(&mut target).fixture("target").as_str(),
        "/api/v1/trusted_publishing/tokens"
    );
}
#[test]
fn malformed_exchange_and_delayed_response_fail_closed() {
    let p = Publisher::GitHub;
    for body in [
        json!({}),
        json!({"token":null}),
        json!({"token":"bad"}),
        json!({"token":"cio_tp_"}),
        json!({"token": "x".repeat(16385)}),
        json!({"token":token_response().get("token").fixture("token"),"scope":"untrusted"}),
    ] {
        let permit = TrustedPublishingPermit::confirm_exchange(
            assertion(p, 100, CredentialOrigin::Production),
            policy(p, 100),
        );
        assert!(decode(permit, &body, 100).is_err());
    }
    for now in [99, 1900, u64::MAX] {
        let permit = TrustedPublishingPermit::confirm_exchange(
            assertion(p, 100, CredentialOrigin::Production),
            policy(p, 100),
        );
        assert!(decode(permit, &token_response(), now).is_err());
    }
    let mut response = token_response();
    let mut text = response
        .get("token")
        .fixture("token")
        .as_str()
        .fixture("token")
        .as_bytes()
        .to_vec();
    *text.last_mut().fixture("last") = b'!';
    *response.get_mut("token").fixture("token") = json!(String::from_utf8(text).fixture("ascii"));
    let permit = TrustedPublishingPermit::confirm_exchange(
        assertion(p, 100, CredentialOrigin::Production),
        policy(p, 100),
    );
    assert!(decode(permit, &response, 100).is_err());
}
