use super::*;
use crate::{
    credentials::{Api, Credential, CredentialKind, CredentialOrigin, TrustedPublishing},
    publishing::{PublishMetadata, PublishRequest, SlicePackage},
    wire::TEST_GATE_LOCK,
};
use alloc::{format, vec::Vec};
mod adapter;
mod failures;
fn complete<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    // Stream drivers cooperatively yield even for immediately ready fixtures.
    for _ in 0..4096 {
        if let Poll::Ready(value) = future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            return value;
        }
    }
    unreachable!("stream fixture exceeded its poll bound")
}
const META: &[u8] = br#"{"name":"serde","vers":"1.2.3","deps":[],"features":{},"authors":[],"description":"example","documentation":null,"homepage":null,"readme":null,"readme_file":null,"keywords":[],"categories":[],"license":"MIT","license_file":null,"repository":null,"badges":{},"links":null,"rust_version":"1.92"}"#;
const RESPONSE: &[u8] = include_bytes!("../../../publishing/publish/success.json");
fn request() -> PublishRequest<'static> {
    PublishRequest::new(
        PublishMetadata::from_json(META).fixture("metadata"),
        5,
        StreamLimits::new(65536, 4096, 65536, 65536, 2).fixture("limits"),
    )
    .fixture("publish")
}
fn framed() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&u32::try_from(META.len()).fixture("length").to_le_bytes());
    bytes.extend_from_slice(META);
    bytes.extend_from_slice(&5_u32.to_le_bytes());
    bytes.extend_from_slice(b"crate");
    bytes
}
fn token<K: CredentialKind>(origin: CredentialOrigin, bytes: &[u8; 32]) -> Credential<K> {
    let mut source = *bytes;
    let token = Credential::from_mut_bytes(origin, &mut source).fixture("credential");
    assert_eq!(source, [0; 32]);
    token
}
fn bytes() -> [u8; 32] {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    [b'a'.saturating_add(
        u8::try_from(NEXT.fetch_add(1, Ordering::Relaxed) % 26).fixture("variation"),
    ); 32]
}
// A local source proves that neither local nor blocking upload requires Send.
struct LocalSource(SlicePackage<'static>, Rc<()>);
impl BlockingStreamSource for LocalSource {
    type Error = crate::publishing::PublishError;
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    fn read_chunk(&mut self, output: &mut [u8]) -> Result<StreamRead, Self::Error> {
        assert_eq!(Rc::strong_count(&self.1), 1);
        BlockingStreamSource::read_chunk(&mut self.0, output)
    }
}
impl LocalAsyncStreamSource for LocalSource {
    type Error = crate::publishing::PublishError;
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    async fn read_chunk_local<'a>(
        &'a mut self,
        output: &'a mut [u8],
    ) -> Result<StreamRead, Self::Error> {
        BlockingStreamSource::read_chunk(self, output)
    }
}
#[test]
fn publication_has_exact_three_mode_wire_and_decoding_witnesses_for_both_credentials() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let bytes = bytes();
    let api = token::<Api>(CredentialOrigin::Production, &bytes);
    let trusted = token::<TrustedPublishing>(CredentialOrigin::Production, &bytes);
    let expected = framed();
    for temporary in [false, true] {
        let authorization = if temporary {
            format!("Bearer {}", core::str::from_utf8(&bytes).fixture("ascii"))
        } else {
            core::str::from_utf8(&bytes).fixture("ascii").into()
        };
        let mut fixture = Fixture::new(Method::Put, "/api/v1/crates/new", &expected, RESPONSE, 200);
        fixture.expected_auth = Some(authorization.as_bytes());
        let local = Local(fixture, Rc::new(()));
        for mode in 0..3 {
            for chunk in [1, 7, 4096] {
                reset_test_gate();
                let client =
                    RegistryClient::production(&local.0, identity(), 16384).fixture("client");
                let local_client =
                    RegistryClient::production(&local, identity(), 16384).fixture("local");
                let mut buffers = Buffers::new();
                let mut scratch = alloc::vec![0xa5; chunk];
                let mut parts = buffers.parts();
                parts.body = &mut scratch;
                let permit = if temporary {
                    request().confirm_trusted(&trusted)
                } else {
                    request().confirm_api(&api)
                };
                let mut source = SlicePackage::new(b"crate");
                let mut local_source = LocalSource(SlicePackage::new(b"crate"), Rc::new(()));
                let value = match mode {
                    0 => client.publish(permit, &mut local_source, parts),
                    1 => complete(local_client.publish_local(permit, &mut local_source, parts)),
                    2 => complete(send(client.publish_async(permit, &mut source, parts))),
                    _ => unreachable!("mode"),
                }
                .fixture("publish");
                assert_eq!(value.krate().name, "serde");
                assert!(
                    value
                        .warnings()
                        .get("other")
                        .fixture("warnings")
                        .fixture("other")
                        .array()
                        .is_ok()
                );
                assert!(scratch.iter().all(|b| *b == 0));
                assert!(buffers.body.iter().all(|b| *b == 0xa5));
                cloud_sdk_sanitization::sanitize_bytes(&mut buffers.body);
                buffers.cleared();
            }
        }
        assert_eq!(local.0.calls.load(Ordering::SeqCst), 9);
    }
    coverage::record(Method::Put, "/api/v1/crates/new");
    reset_test_gate();
}
