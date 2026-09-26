use super::*;
use crate::{
    accounts::AccountRequest,
    accounts::cargo::CargoOwnersRequest,
    catalog::CatalogRequest,
    credentials::{ApiToken, CredentialContext, CredentialOrigin},
    query::{Parameter, PerPage, SearchQuery},
    wire::TEST_GATE_LOCK,
};

const OWNERS: &[u8] = br#"{"users":[{"id":70,"login":"github:rust-lang:core","name":"Core"}]}"#;

#[test]
fn explicit_cargo_owners_execute_the_minimal_profile_in_all_modes() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let expected = [b'a'.saturating_add(
        u8::try_from(NEXT.fetch_add(1, Ordering::Relaxed) % 26).fixture("variation"),
    ); 32];
    let mut source = expected;
    let token =
        ApiToken::from_mut_bytes(CredentialOrigin::Production, &mut source).fixture("token");
    assert!(source.iter().all(|b| *b == 0));
    let mut fixture = Fixture::new(Method::Get, "/api/v1/crates/serde/owners", b"", OWNERS, 200);
    fixture.expected_auth = Some(&expected);
    parity(
        || CargoOwnersRequest::new(super::operations::name(), &token),
        fixture,
    );
    let request = CargoOwnersRequest::new(super::operations::name(), &token);
    let mut target = [0; 128];
    let target = request.write_target(&mut target).fixture("target");
    // The Cargo override must not widen the generic website token allowlist.
    assert!(CredentialContext::api(CredentialOrigin::Production, Method::Get, target).is_err());
}

#[test]
fn cargo_owner_preflight_and_response_failures_clear_all_scratch() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    for mode in 0..3 {
        for fault in 0..9 {
            reset_test_gate();
            let mut token = super::operations::token(if fault == 0 {
                CredentialOrigin::Staging
            } else {
                CredentialOrigin::Production
            });
            if fault == 1 {
                token.clear();
            }
            let mut fixture =
                Fixture::new(Method::Get, "/api/v1/crates/serde/owners", b"", OWNERS, 200);
            match fault {
                4 => fixture.wire = br#"{"users":[{}]}"#,
                5 => fixture.media = false,
                6 => fixture.status = 403,
                7 => fixture.fail = true,
                8 => fixture.encoding = Some(b"gzip"),
                _ => {}
            }
            let local = Local(fixture, Rc::new(()));
            let client = RegistryClient::production(&local.0, identity(), 16384).fixture("client");
            let local_client =
                RegistryClient::production(&local, identity(), 16384).fixture("local");
            if fault == 2 {
                local.0.changed.store(true, Ordering::SeqCst);
            }
            let mut buffers = Buffers::new();
            let mut parts = buffers.parts();
            if fault == 3 {
                parts.credential = parts.credential.get_mut(..1).fixture("short scratch");
            }
            let request = CargoOwnersRequest::new(super::operations::name(), &token);
            let result = match mode {
                0 => client.execute(request, parts),
                1 => ready(local_client.execute_local(request, parts)),
                2 => ready(send(client.execute_async(request, parts))),
                _ => unreachable!("mode"),
            };
            assert!(result.is_err());
            assert_eq!(
                local.0.calls.load(Ordering::SeqCst),
                usize::from(fault >= 4)
            );
            if fault == 3 {
                assert_eq!(buffers.credential.first(), Some(&0));
                assert!(
                    buffers
                        .credential
                        .get(1..)
                        .fixture("unused scratch")
                        .iter()
                        .all(|b| *b == 0xa5)
                );
                cloud_sdk_sanitization::sanitize_bytes(&mut buffers.credential);
            } // This region was not passed.
            buffers.cleared();
            if fault < 4 {
                drop(
                    crate::wire::OfficialApiGate::new(identity())
                        .begin()
                        .fixture("preflight must not consume rate admission"),
                );
            }
        }
    }
    reset_test_gate();
}

#[test]
fn cargo_owner_unpolled_and_pending_cancellation_clear_scratch() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let token = super::operations::token(CredentialOrigin::Production);
    for local_mode in [false, true] {
        for poll in [false, true] {
            reset_test_gate();
            let mut fixture =
                Fixture::new(Method::Get, "/api/v1/crates/serde/owners", b"", OWNERS, 200);
            fixture.pending = true;
            let local = Local(fixture, Rc::new(()));
            let client = RegistryClient::production(&local.0, identity(), 16384).fixture("client");
            let local_client =
                RegistryClient::production(&local, identity(), 16384).fixture("local");
            let mut buffers = Buffers::new();
            let request = CargoOwnersRequest::new(super::operations::name(), &token);
            if local_mode {
                let mut future =
                    core::pin::pin!(local_client.execute_local(request, buffers.parts()));
                if poll {
                    assert!(
                        future
                            .as_mut()
                            .poll(&mut Context::from_waker(Waker::noop()))
                            .is_pending()
                    );
                }
            } else {
                let mut future =
                    core::pin::pin!(send(client.execute_async(request, buffers.parts())));
                if poll {
                    assert!(
                        future
                            .as_mut()
                            .poll(&mut Context::from_waker(Waker::noop()))
                            .is_pending()
                    );
                }
            }
            buffers.cleared();
            assert_eq!(local.0.calls.load(Ordering::SeqCst), usize::from(poll));
        }
    }
    reset_test_gate();
}

// Independent Cargo Registry Web API profile, not generated from OpenAPI or
// the provider serializer. Keep the minimal response free of website fields.
#[test]
fn cargo_search_minimal_response_and_literal_query_have_three_mode_parity() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let params = [
        Parameter::Search(SearchQuery::new("serde json").fixture("query")),
        Parameter::PerPage(PerPage::new(10).fixture("per page")),
    ];
    let mut fixture = Fixture::new(
        Method::Get, "/api/v1/crates?per_page=10&q=serde%20json", b"",
        br#"{"crates":[{"name":"serde","max_version":"1.0.0","description":"fixture"}],"meta":{"total":1}}"#, 200,
    );
    fixture.auth = false;
    parity(
        || CatalogRequest::cargo_search(&params).fixture("Cargo search"),
        fixture,
    );
}

#[test]
fn website_owner_decoder_does_not_silently_claim_the_minimal_cargo_profile() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    for mode in 0..3 {
        reset_test_gate();
        let mut fixture = Fixture::new(
            Method::Get,
            "/api/v1/crates/serde/owners",
            b"",
            br#"{"users":[{"id":70,"login":"github:rust-lang:core","name":"Core"}]}"#,
            200,
        );
        fixture.auth = false;
        let client = RegistryClient::production(&fixture, identity(), 16384).fixture("client");
        let mut buffers = Buffers::new();
        let request = AccountRequest::owners(super::operations::name());
        let result = match mode {
            0 => client.execute(request, buffers.parts()),
            1 => ready(client.execute_local(request, buffers.parts())),
            2 => ready(send(client.execute_async(request, buffers.parts()))),
            _ => unreachable!("mode"),
        };
        assert!(matches!(result, Err(DiscoveryExecutionError::Model(_))));
        assert_eq!(fixture.calls.load(Ordering::SeqCst), 1);
        buffers.cleared();
    }
    reset_test_gate();
}
