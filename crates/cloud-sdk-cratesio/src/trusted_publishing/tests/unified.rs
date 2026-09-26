use super::*;
use crate::{
    client::async_test_support::{Fixture, parity},
    wire::TEST_GATE_LOCK,
};
use cloud_sdk::Method;
#[test]
fn all_trusted_publishing_operations_have_three_mode_parity() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let token = api(CredentialOrigin::Production);
    let params = [Parameter::Crate(CrateName::new("regex").fixture("crate"))];
    for publisher in [Publisher::GitHub, Publisher::GitLab] {
        let provider = if publisher == Publisher::GitHub {
            "github"
        } else {
            "gitlab"
        };
        let q = Query::new(publisher.query(), &params).fixture("query");
        let mut value = fixture(&format!("list_trustpub_{provider}_configs"));
        *value.pointer_mut("/meta/next_page").fixture("next") = Value::Null;
        let wire = serde_json::to_vec(&value).fixture("response");
        let mut target = [0; 4096];
        let list = TrustedPublishingPermit::list(publisher, q, &token).fixture("list");
        let path = list.write_target(&mut target).fixture("target");
        parity(
            || TrustedPublishingPermit::list(publisher, q, &token).fixture("permit"),
            Fixture::new(Method::Get, path.as_str(), b"", &wire, 200),
        );
        let create = TrustedPublishingPermit::confirm_create(config(publisher), &token);
        let path = create.write_target(&mut target).fixture("target");
        let mut payload = [0; 4096];
        let len = create.body(&mut payload).fixture("body");
        let wire = serde_json::to_vec(&fixture(&format!("create_trustpub_{provider}_config")))
            .fixture("response");
        parity(
            || TrustedPublishingPermit::confirm_create(config(publisher), &token),
            Fixture::new(
                Method::Post,
                path.as_str(),
                payload.get(..len).fixture("payload"),
                &wire,
                200,
            ),
        );
        let delete = TrustedPublishingPermit::confirm_delete(publisher, id(42), &token);
        let path = delete.write_target(&mut target).fixture("target");
        parity(
            || TrustedPublishingPermit::confirm_delete(publisher, id(42), &token),
            Fixture::new(Method::Delete, path.as_str(), b"", b"", 204),
        );
    }
    let time = execution::now();
    let assertion = assertion(Publisher::GitHub, time, CredentialOrigin::Production);
    let mut scratch = [0; 32768];
    let payload = assertion
        .with_material_for_adapter(
            &crate::credentials::CredentialContext::exchange(CredentialOrigin::Production),
            &execution::Executor,
            &mut scratch,
            |_, m| m.json_body().fixture("JSON assertion").to_vec(),
        )
        .fixture("material");
    let wire = serde_json::to_vec(&token_response()).fixture("response");
    let mut exchange = Fixture::new(
        Method::Post,
        "/api/v1/trusted_publishing/tokens",
        &payload,
        &wire,
        200,
    );
    exchange.auth = false;
    parity(
        || {
            TrustedPublishingPermit::confirm_exchange(
                super::assertion(Publisher::GitHub, time, CredentialOrigin::Production),
                policy(Publisher::GitHub, time),
            )
        },
        exchange,
    );
    parity(
        || TrustedPublishingPermit::confirm_revoke(temporary(time)),
        Fixture::new(
            Method::Delete,
            "/api/v1/trusted_publishing/tokens",
            b"",
            b"",
            204,
        ),
    );
}
