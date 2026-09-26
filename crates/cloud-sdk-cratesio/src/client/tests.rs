use super::*;
use crate::{
    credentials::{ApiToken, CredentialOrigin},
    discovery::tests::Fixture as _,
    endpoint::OfficialCratesIoEndpoint,
    identifiers::{CrateName, Version},
    publishing::YankRequest,
    wire::{TEST_GATE_LOCK, reset_test_gate},
};
use cloud_sdk::{Method, transport::*};
use core::cell::Cell;

const UA: &str = "registry-test/1 (test@example.org)";
struct Transport {
    calls: Cell<usize>,
    changed: Cell<bool>,
    authorized: bool,
    fail: bool,
    method: Method,
    target: &'static str,
    response: &'static [u8],
}
impl BoundUserAgent for Transport {
    fn configured_user_agent(&self) -> &[u8] {
        UA.as_bytes()
    }
}
impl BoundTransport for Transport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        let host = if self.changed.get() {
            "wrong.invalid"
        } else {
            "crates.io"
        };
        EndpointIdentity::new(EndpointScheme::Https, host, 443, "/")
    }
}
impl Transport {
    fn send(
        &self,
        request: TransportRequest<'_>,
        writer: &mut ResponseWriter<'_>,
    ) -> Result<(), ()> {
        self.calls
            .set(self.calls.get().checked_add(1).fixture("call count"));
        assert_eq!(request.method(), self.method);
        assert_eq!(request.target().as_str(), self.target);
        assert!(request.body().is_empty());
        assert!(request.headers().get("authorization").is_none());
        if self.fail {
            return Err(());
        }
        let mut attempt = writer.begin_attempt().map_err(|_| ())?;
        attempt
            .body_mut()
            .map_err(|_| ())?
            .get_mut(..self.response.len())
            .ok_or(())?
            .copy_from_slice(self.response);
        attempt
            .headers_mut()
            .map_err(|_| ())?
            .try_push(
                "content-type",
                b"application/json",
                HeaderSensitivity::Public,
            )
            .map_err(|_| ())?;
        attempt
            .commit(StatusCode::OK, self.response.len(), ResponseMetadata::EMPTY)
            .map_err(|_| ())
    }
}
impl BlockingRawHttpExecutor for Transport {
    type Error = ();
    fn execute(
        &self,
        request: TransportRequest<'_>,
        _: RawResponsePolicy<'_>,
        writer: &mut ResponseWriter<'_>,
    ) -> Result<(), ()> {
        assert!(!self.authorized);
        self.send(request, writer)
    }
}
impl BlockingAuthorizedRawHttpExecutor for Transport {
    fn execute_authorized(
        &self,
        expected: EndpointIdentity<'_>,
        authorization: HeaderValue<'_>,
        request: TransportRequest<'_>,
        _: RawResponsePolicy<'_>,
        writer: &mut ResponseWriter<'_>,
    ) -> Result<(), ()> {
        assert!(self.authorized);
        assert_eq!(
            expected,
            OfficialCratesIoEndpoint::production_api()
                .identity()
                .fixture("origin")
        );
        assert_eq!(authorization.as_str().len(), 32);
        assert!(!authorization.as_str().starts_with("Bearer "));
        self.send(request, writer)
    }
}
fn transport() -> Transport {
    Transport {
        calls: Cell::new(0),
        changed: Cell::new(false),
        authorized: true,
        fail: false,
        method: Method::Delete,
        target: "/api/v1/crates/serde/1.0.0/yank",
        response: br#"{"ok":true}"#,
    }
}
fn identity() -> IdentifyingUserAgent<'static> {
    IdentifyingUserAgent::new(UA).fixture("identity")
}
fn token(origin: CredentialOrigin) -> ApiToken {
    use core::sync::atomic::{AtomicU8, Ordering};
    static NEXT: AtomicU8 = AtomicU8::new(0);
    let mut bytes = [b'a'.saturating_add(NEXT.fetch_add(1, Ordering::Relaxed) % 26); 32];
    ApiToken::from_mut_bytes(origin, &mut bytes).fixture("token")
}
fn request() -> YankRequest<'static> {
    YankRequest::yank(
        CrateName::new("serde").fixture("crate"),
        Version::new("1.0.0").fixture("version"),
    )
}
#[test]
fn unified_yank_checks_consent_origin_response_and_cleans_all_scratch() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate lock");
    for scenario in 0..6 {
        reset_test_gate();
        let fixture = Transport {
            fail: scenario == 2,
            response: if scenario == 3 {
                br#"{"ok":false}"#
            } else {
                br#"{"ok":true}"#
            },
            ..transport()
        };
        let client = RegistryClient::production(&fixture, identity(), 4096).fixture("client");
        fixture.changed.set(scenario == 1);
        let token = token(if scenario == 4 {
            CredentialOrigin::Staging
        } else {
            CredentialOrigin::Production
        });
        let occupied = if scenario == 5 {
            Some(
                crate::wire::OfficialApiGate::new(identity())
                    .begin()
                    .fixture("gate"),
            )
        } else {
            None
        };
        let mut credential = [0xa5; 1024];
        let mut body = [0xa5; 1024];
        let mut response = [0xa5; 4096];
        let mut headers = [0xa5; 512];
        let result = client.execute(
            request().confirm(&token),
            RegistryBuffers {
                credential: &mut credential,
                body: &mut body,
                response: &mut response,
                headers: &mut headers,
            },
        );
        assert_eq!(result.is_ok(), scenario == 0);
        assert_eq!(
            fixture.calls.get(),
            usize::from(matches!(scenario, 0 | 2 | 3))
        );
        for bytes in [&credential[..], &body[..], &response[..], &headers[..]] {
            assert!(bytes.iter().all(|byte| *byte == 0));
        }
        drop(occupied);
    }
    reset_test_gate();
}

#[test]
fn unified_reads_keep_anonymous_authority_and_shared_admission() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate lock");
    reset_test_gate();
    let fixture = Transport {
        authorized: false,
        method: Method::Get,
        target: "/api/v1/site_metadata",
        response: include_bytes!("../discovery/fixtures/get_site_metadata.json"),
        ..transport()
    };
    let first = RegistryClient::production(&fixture, identity(), 4096).fixture("client");
    let second = RegistryClient::production(&fixture, identity(), 4096).fixture("client");
    let mut credential = [0xa5; 32];
    let mut body = [0xa5; 32];
    let mut response = [0xa5; 4096];
    let mut headers = [0xa5; 512];
    for (index, client) in [&first, &second].into_iter().enumerate() {
        let result = client.execute(
            crate::discovery::DiscoveryRequest::site_metadata(),
            RegistryBuffers {
                credential: &mut credential,
                body: &mut body,
                response: &mut response,
                headers: &mut headers,
            },
        );
        if index == 0 {
            assert!(result.is_ok());
        } else {
            assert!(matches!(result, Err(DiscoveryExecutionError::Schedule(_))));
        }
        for bytes in [&credential[..], &body[..], &response[..], &headers[..]] {
            assert!(bytes.iter().all(|byte| *byte == 0));
        }
    }
    assert_eq!(fixture.calls.get(), 1);
    reset_test_gate();
}

#[test]
fn wrong_origin_secret_path_permits_fail_before_dispatch_and_clear_all_scratch() {
    use crate::{
        accounts::personal::PersonalPermit,
        credentials::{EmailConfirmationToken, OwnerInvitationToken},
        identifiers::NumericId,
    };
    use core::sync::atomic::{AtomicU8, Ordering};
    static NEXT: AtomicU8 = AtomicU8::new(0);
    let _lock = TEST_GATE_LOCK.lock().fixture("gate lock");
    reset_test_gate();
    let transport = transport();
    let client = RegistryClient::production(&transport, identity(), 4096).fixture("client");
    for invitation in [false, true] {
        reset_test_gate();
        let mut source = [b'a'.saturating_add(NEXT.fetch_add(1, Ordering::Relaxed) % 26); 24];
        let permit = if invitation {
            PersonalPermit::accept_invitation_token(
                OwnerInvitationToken::from_mut_bytes(CredentialOrigin::Staging, &mut source)
                    .fixture("invitation token"),
                NumericId::new(42).fixture("crate ID"),
            )
        } else {
            PersonalPermit::confirm_email(
                EmailConfirmationToken::from_mut_bytes(CredentialOrigin::Staging, &mut source)
                    .fixture("confirmation token"),
            )
        };
        assert!(source.iter().all(|byte| *byte == 0));
        let mut credential = [0xa5; 1024];
        let mut body = [0xa5; 1024];
        let mut response = [0xa5; 4096];
        let mut headers = [0xa5; 512];
        // Call the trait directly too: it must not bypass the facade's cleanup.
        let result = BlockingRegistryOperation::run(
            permit,
            &client,
            RegistryBuffers {
                credential: &mut credential,
                body: &mut body,
                response: &mut response,
                headers: &mut headers,
            },
        );
        assert!(matches!(
            result,
            Err(DiscoveryExecutionError::Model(DiscoveryError::Binding))
        ));
        assert_eq!(transport.calls.get(), 0);
        for bytes in [&credential[..], &body[..], &response[..], &headers[..]] {
            assert!(bytes.iter().all(|byte| *byte == 0));
        }
        let _admission = crate::wire::OfficialApiGate::new(identity())
            .begin()
            .fixture("unused gate");
    }
    reset_test_gate();
}
