use core::cell::Cell;

use cloud_sdk::authentication::{
    AuthenticatedRequest, BlockingAuthenticatedTransport, BoundCredentialTransport,
    CREDENTIAL_BINDING_BYTES, CredentialBinding,
};
use cloud_sdk::operation::{ExecutionPermitError, PreparationStorage, PreparedExecutionError};
use cloud_sdk::transport::{
    BoundTransport, EndpointIdentity, EndpointIdentityError, ResponseWriter,
};
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, MockError, MockExchange, MockTransport, ResponseFixture,
};

use super::{
    CURRENCY, RobotCatalogPlanError, RobotOrderCurrencyRequest, RobotStandardOrderPlan, STANDARD,
    decode_currency_fixture, decode_standard,
};
use crate::endpoint::official_robot_endpoint_identity;

struct RotatingCredentialTransport<'a> {
    inner: MockTransport<'a>,
    rotated: Cell<bool>,
    before: CredentialBinding,
    after: CredentialBinding,
}

impl BlockingAuthenticatedTransport for RotatingCredentialTransport<'_> {
    type Error = MockError;

    fn send_authenticated(
        &self,
        request: AuthenticatedRequest<'_, '_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        let result = self.inner.send_authenticated(request, response);
        self.rotated.set(true);
        result
    }
}

impl BoundTransport for RotatingCredentialTransport<'_> {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.inner.endpoint_identity()
    }
}

impl BoundCredentialTransport for RotatingCredentialTransport<'_> {
    fn credential_binding(&self) -> CredentialBinding {
        if self.rotated.get() {
            self.after
        } else {
            self.before
        }
    }
}

#[test]
fn observed_catalog_execution_rejects_credential_rotation() {
    let request = RobotOrderCurrencyRequest::new();
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("currency preparation failed"));
    let transport_request = prepared.as_untyped().transport_request();
    let expected = ExpectedRequest::new(transport_request.method(), transport_request.target())
        .with_body(transport_request.body())
        .with_headers(transport_request.headers());
    let fixture = ResponseFixture::success(
        FixtureBody::new(CURRENCY).unwrap_or_else(|_| unreachable!("currency fixture failed")),
    )
    .with_content_type("application/json");
    let exchanges = [MockExchange::new(expected, fixture)];
    let endpoint = official_robot_endpoint_identity()
        .unwrap_or_else(|_| unreachable!("Robot endpoint failed"));
    let before = credential(0x31);
    let transport = RotatingCredentialTransport {
        inner: MockTransport::new(&exchanges)
            .with_endpoint(endpoint)
            .with_credential_binding(before),
        rotated: Cell::new(false),
        before,
        after: credential(0x32),
    };
    let mut response_body = [0_u8; CURRENCY.len()];
    let mut response_headers = [0_u8; 128];

    let error = prepared
        .execute_observed_blocking(&transport, &mut response_body, &mut response_headers)
        .err()
        .unwrap_or_else(|| unreachable!("credential rotation was accepted"));

    assert!(
        matches!(
            error,
            PreparedExecutionError::AuthorizationInvalid(ExecutionPermitError::CredentialMismatch)
        ),
        "unexpected execution error: {error:?}"
    );
    assert!(transport.inner.is_complete());
}

#[test]
fn catalog_plan_rejects_different_observation_credentials() {
    let product = super::CredentialObserved::from_parts(
        decode_standard(STANDARD).unwrap_or_else(|_| unreachable!("standard fixture failed")),
        credential(0x41),
    );
    let currency = super::CredentialObserved::from_parts(
        decode_currency_fixture(CURRENCY)
            .unwrap_or_else(|_| unreachable!("currency fixture failed")),
        credential(0x42),
    );

    assert_eq!(
        RobotStandardOrderPlan::new(&product, &currency, 0, 0, 0, &[]).err(),
        Some(RobotCatalogPlanError::CredentialMismatch)
    );
}

fn credential(byte: u8) -> CredentialBinding {
    CredentialBinding::new([byte; CREDENTIAL_BINDING_BYTES])
        .unwrap_or_else(|_| unreachable!("credential fixture is nonzero"))
}
