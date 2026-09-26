use super::execution::{Executor, client, now, stage};
use super::*;
use crate::{
    publishing::{PublishBuffers, PublishClient, SlicePackage},
    wire::{IdentifyingUserAgent, TEST_GATE_LOCK, reset_test_gate},
};
use cloud_sdk::transport::{BlockingStreamSink, StreamPartialState};
#[derive(Default)]
struct Sink {
    committed: bool,
}
impl BlockingStreamSink for Sink {
    type Error = ();
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<usize, ()> {
        Ok(bytes.len())
    }
    fn commit(&mut self) -> Result<(), ()> {
        self.committed = true;
        Ok(())
    }
    fn abort(&mut self, _: StreamPartialState) {
        self.committed = false;
    }
}
#[test]
fn gitlab_exchange_publish_then_explicit_revoke() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let client = client();
    let time = now();
    let mut secret = [0; 32768];
    let mut body = [0; 4096];
    let mut response = [0; 8192];
    let mut headers = [0; 512];
    reset_test_gate();
    let result = client
        .execute(
            TrustedPublishingPermit::confirm_exchange(
                assertion(Publisher::GitLab, time, CredentialOrigin::Production),
                policy(Publisher::GitLab, time),
            ),
            TrustedPublishingBuffers {
                credential: &mut secret,
                body: &mut body,
                response: &mut response,
                headers: &mut headers,
            },
            |_, material, _, _, writer| {
                assert!(material.authorization().is_none());
                stage(
                    writer,
                    200,
                    &serde_json::to_vec(&token_response()).fixture("body"),
                    Some(b"application/json"),
                    None,
                    None,
                );
                Ok::<_, ()>(())
            },
        )
        .fixture("exchange");
    let TrustedPublishingResponse::Exchanged(token) = result else {
        unreachable!("variant")
    };
    let metadata = super::tokens::metadata("regex");
    let permit = token
        .confirm_publish(super::tokens::publish(&metadata), now())
        .fixture("permit");
    let publish = PublishClient::production(
        &Executor,
        IdentifyingUserAgent::new("tests/1 (tests@example.org)").fixture("agent"),
        8192,
    )
    .fixture("publisher");
    let mut success: Value =
        serde_json::from_str(include_str!("../../publishing/publish/success.json"))
            .fixture("fixture");
    let record = success.get_mut("crate").fixture("crate");
    *record.get_mut("name").fixture("name") = json!("regex");
    *record.get_mut("id").fixture("id") = json!("regex");
    let wire = serde_json::to_vec(&success).fixture("wire");
    reset_test_gate();
    let mut source = SlicePackage::new(b"x");
    let mut sink = Sink::default();
    let mut scratch = [0xa5; 64];
    publish
        .execute(
            permit,
            &mut source,
            PublishBuffers {
                credential: &mut secret,
                response: &mut response,
                headers: &mut headers,
            },
            |_, material, _, upload, _, writer| {
                assert!(material.authorization().is_some());
                upload
                    .transfer_to(&mut sink, &mut scratch)
                    .fixture("transfer");
                stage(writer, 200, &wire, Some(b"application/json"), None, None);
                Ok::<_, ()>(())
            },
        )
        .fixture("publish");
    assert!(sink.committed);
    assert_eq!(scratch, [0; 64]);
    reset_test_gate();
    let result = client
        .execute(
            TrustedPublishingPermit::confirm_revoke(token),
            TrustedPublishingBuffers {
                credential: &mut secret,
                body: &mut body,
                response: &mut response,
                headers: &mut headers,
            },
            |_, material, request, _, writer| {
                assert!(material.authorization().is_some());
                assert_eq!(request.method(), cloud_sdk::Method::Delete);
                stage(writer, 204, b"", None, None, None);
                Ok::<_, ()>(())
            },
        )
        .fixture("revoke");
    assert!(matches!(result, TrustedPublishingResponse::Revoked));
    for bytes in [&secret[..], &body[..], &response[..], &headers[..]] {
        assert!(bytes.iter().all(|b| *b == 0));
    }
}
