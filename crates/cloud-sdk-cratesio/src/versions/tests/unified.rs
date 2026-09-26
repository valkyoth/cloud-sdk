use super::*;
use crate::{
    client::async_test_support::{Fixture as WireFixture, parity},
    wire::TEST_GATE_LOCK,
};
#[test]
fn all_versions_requests_execute_through_unified_client() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    assert_eq!(requests().len(), FIXTURES.len());
    for (request, wire) in requests().into_iter().zip(FIXTURES) {
        let mut target = [0; 4096];
        let target = request.write_target(&mut target).fixture("target");
        let mut transport = WireFixture::new(
            cloud_sdk::Method::Get,
            target.as_str(),
            b"",
            wire.as_bytes(),
            200,
        );
        transport.auth = false;
        parity(|| request, transport);
    }
}
