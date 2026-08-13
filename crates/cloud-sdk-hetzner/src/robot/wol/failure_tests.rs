use super::RobotWolFailureCode;
use super::failure::classify;

#[test]
fn source_locked_failures_are_operation_bound() {
    assert_eq!(
        classify(false, 404, "SERVER_NOT_FOUND"),
        Some(RobotWolFailureCode::ServerNotFound)
    );
    assert_eq!(
        classify(true, 404, "WOL_NOT_AVAILABLE"),
        Some(RobotWolFailureCode::WolNotAvailable)
    );
    assert_eq!(
        classify(true, 500, "WOL_FAILED"),
        Some(RobotWolFailureCode::WolFailed)
    );
    assert_eq!(classify(false, 500, "WOL_FAILED"), None);
    assert_eq!(classify(true, 500, "RESET_FAILED"), None);
    assert_eq!(classify(true, 404, "NOT_FOUND"), None);
}
