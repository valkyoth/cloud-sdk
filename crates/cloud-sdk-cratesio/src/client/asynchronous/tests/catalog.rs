use super::operations::{name, token};
use super::*;
use crate::{
    catalog::CatalogRequest, credentials::CredentialOrigin, query::Parameter, wire::TEST_GATE_LOCK,
};
#[test]
fn token_catalog_executes_only_list_and_clears_unpolled_futures() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let token = token(CredentialOrigin::Production);
    let params = [Parameter::Following];
    let request = CatalogRequest::list(&params).fixture("following");
    let mut target = [0; 4096];
    let path = request.write_target(&mut target).fixture("target");
    for local_mode in [false, true] {
        reset_test_gate();
        let local = Local(
            Fixture::new(
                Method::Get,
                path.as_str(),
                b"",
                include_bytes!("../../../catalog/fixtures/list_crates.json"),
                200,
            ),
            Rc::new(()),
        );
        let client = RegistryClient::production(&local.0, identity(), 16384).fixture("client");
        let local_client = RegistryClient::production(&local, identity(), 16384).fixture("local");
        let mut buffers = Buffers::new();
        if local_mode {
            drop(local_client.catalog_with_token_local(request, &token, buffers.parts()));
        } else {
            drop(send(client.catalog_with_token_async(
                request,
                &token,
                buffers.parts(),
            )));
        }
        buffers.cleared();
        assert_eq!(local.0.calls.load(Ordering::SeqCst), 0);
        let rejected = CatalogRequest::crate_metadata(name(), &[]).fixture("metadata");
        let result = if local_mode {
            ready(local_client.catalog_with_token_local(rejected, &token, buffers.parts()))
        } else {
            ready(send(client.catalog_with_token_async(
                rejected,
                &token,
                buffers.parts(),
            )))
        };
        assert!(result.is_err());
        buffers.cleared();
        assert_eq!(local.0.calls.load(Ordering::SeqCst), 0);
        let result = if local_mode {
            ready(local_client.catalog_with_token_local(request, &token, buffers.parts()))
        } else {
            ready(send(client.catalog_with_token_async(
                request,
                &token,
                buffers.parts(),
            )))
        };
        assert!(result.is_ok());
        buffers.cleared();
        assert_eq!(local.0.calls.load(Ordering::SeqCst), 1);
    }
    reset_test_gate();
}
