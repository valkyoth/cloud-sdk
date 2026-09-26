use super::*;
use crate::{
    accounts::AccountRequest,
    catalog::CatalogRequest,
    query::{Parameter, PerPage, SearchQuery},
    wire::TEST_GATE_LOCK,
};

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
