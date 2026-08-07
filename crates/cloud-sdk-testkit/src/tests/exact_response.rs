use cloud_sdk::transport::{DeliveryClassified, DeliveryPhase, StatusCode};

use crate::{FixtureBody, MockError, ResponseFixture, ResponseFixtureError};

#[test]
fn exact_success_status_and_delivery_classification_support_permit_fixtures() {
    let body = FixtureBody::new(b"");
    assert!(body.is_ok());
    let Ok(body) = body else {
        unreachable!("testkit fixture body construction failed")
    };
    let created = ResponseFixture::success_at(StatusCode::CREATED, body);
    let no_content = ResponseFixture::success_at(StatusCode::NO_CONTENT, body);
    let rejected = ResponseFixture::success_at(StatusCode::TOO_MANY_REQUESTS, body);
    assert!(created.is_ok());
    assert!(no_content.is_ok());
    assert!(matches!(
        rejected,
        Err(ResponseFixtureError::NonSuccessStatus)
    ));
    assert_eq!(
        MockError::Exhausted.delivery_phase(),
        DeliveryPhase::NotSent
    );
}
