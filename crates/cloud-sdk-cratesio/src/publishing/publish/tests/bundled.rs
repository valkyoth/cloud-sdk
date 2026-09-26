use super::*;
use crate::{
    bundled,
    client::{RegistryBuffers, RegistryClient},
    credentials::{Api, Credential, CredentialKind, CredentialOrigin, TrustedPublishing},
    wire::IdentifyingUserAgent,
};
fn identity() -> IdentifyingUserAgent<'static> {
    IdentifyingUserAgent::new("publish-test/1 (test@example.org)").fixture("identity")
}
fn timeouts() -> bundled::RequestTimeouts {
    bundled::RequestTimeouts::new(
        core::time::Duration::from_secs(5),
        core::time::Duration::from_secs(2),
    )
    .fixture("timeouts")
}
fn token<K: CredentialKind>() -> Credential<K> {
    use core::sync::atomic::{AtomicU8, Ordering};
    static NEXT: AtomicU8 = AtomicU8::new(0);
    let mut bytes = [b'a'.saturating_add(NEXT.fetch_add(1, Ordering::Relaxed) % 26); 32];
    Credential::from_mut_bytes(CredentialOrigin::Production, &mut bytes).fixture("token")
}
struct Buffers {
    credential: [u8; 1024],
    body: [u8; 16],
    response: [u8; 1024],
    headers: [u8; 256],
}
impl Buffers {
    fn new() -> Self {
        Self {
            credential: [0xa5; 1024],
            body: [0xa5; 16],
            response: [0xa5; 1024],
            headers: [0xa5; 256],
        }
    }
    fn parts(&mut self) -> RegistryBuffers<'_> {
        RegistryBuffers {
            credential: &mut self.credential,
            body: &mut self.body,
            response: &mut self.response,
            headers: &mut self.headers,
        }
    }
    fn cleared(&self) {
        for bytes in [
            &self.credential[..],
            &self.body[..],
            &self.response[..],
            &self.headers[..],
        ] {
            assert!(bytes.iter().all(|b| *b == 0));
        }
    }
}
#[cfg(feature = "blocking-rustls")]
#[test]
fn bundled_blocking_publish_rejects_wrong_origin_and_clears_every_buffer() {
    let transport = bundled::staging_blocking(identity(), timeouts()).fixture("transport");
    let client = RegistryClient::staging(&transport, identity(), 1024).fixture("client");
    let api = token::<Api>();
    let trusted = token::<TrustedPublishing>();
    for temporary in [false, true] {
        let mut buffers = Buffers::new();
        let mut source = SlicePackage::new(b"crate");
        let permit = if temporary {
            request(5).confirm_trusted(&trusted)
        } else {
            request(5).confirm_api(&api)
        };
        assert!(
            client
                .publish(permit, &mut source, buffers.parts())
                .is_err()
        );
        buffers.cleared();
        let mut untouched = [0; 5];
        assert_eq!(source.read_chunk(&mut untouched), Ok(StreamRead::Chunk(5)));
        assert_eq!(&untouched, b"crate");
    }
}
#[cfg(feature = "async-rustls")]
#[test]
fn bundled_async_publish_arms_cleanup_before_polling_and_origin_checks() {
    let transport = bundled::staging_async(identity(), timeouts()).fixture("transport");
    let client = RegistryClient::staging(&transport, identity(), 1024).fixture("client");
    let api = token::<Api>();
    let trusted = token::<TrustedPublishing>();
    fn send<F: core::future::Future + Send>(f: F) -> F {
        f
    }
    for temporary in [false, true] {
        for local in [false, true] {
            for polled in [false, true] {
                let mut buffers = Buffers::new();
                let mut source = SlicePackage::new(b"crate");
                let permit = if temporary {
                    request(5).confirm_trusted(&trusted)
                } else {
                    request(5).confirm_api(&api)
                };
                if local {
                    let future = client.publish_local(permit, &mut source, buffers.parts());
                    if polled {
                        assert!(super::asynchronous::ready(future).is_err());
                    } else {
                        drop(future);
                    }
                } else {
                    let future = send(client.publish_async(permit, &mut source, buffers.parts()));
                    if polled {
                        assert!(super::asynchronous::ready(future).is_err());
                    } else {
                        drop(future);
                    }
                }
                buffers.cleared();
                let mut untouched = [0; 5];
                assert_eq!(source.read_chunk(&mut untouched), Ok(StreamRead::Chunk(5)));
                assert_eq!(&untouched, b"crate");
            }
        }
    }
}
