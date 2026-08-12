use alloc::format;

use cloud_sdk::operation::PermitTimestamp;

use super::tests::{DETAIL, decode_get, subnet};
use super::{
    RobotSubnetEvidenceError, RobotSubnetGetRequest, RobotSubnetMutationLease,
    RobotSubnetObservationWindow, RobotSubnetTrafficUpdate,
};

#[test]
fn destructive_evidence_and_traffic_diagnostics_fail_closed() {
    assert!(matches!(
        RobotSubnetObservationWindow::new(
            PermitTimestamp::from_seconds(100),
            PermitTimestamp::from_seconds(130),
        ),
        Err(RobotSubnetEvidenceError::ObservationWindowTooWide)
    ));
    assert!(matches!(
        RobotSubnetMutationLease::new(
            subnet("2001:db8::"),
            b"",
            PermitTimestamp::from_seconds(130),
        ),
        Err(RobotSubnetEvidenceError::InvalidLockIdentity)
    ));
    let lease = RobotSubnetMutationLease::new(
        subnet("2001:db8:1::"),
        b"test-lock-generation-0001",
        PermitTimestamp::from_seconds(130),
    )
    .unwrap_or_else(|_| unreachable!("mutation lease failed"));
    assert_eq!(
        lease.covers(&subnet("2001:db8::"), PermitTimestamp::from_seconds(129)),
        Err(RobotSubnetEvidenceError::LockResourceMismatch)
    );

    let update = RobotSubnetTrafficUpdate::warnings(true)
        .with_hourly(50)
        .with_daily(500)
        .with_monthly(8);
    assert_eq!(
        format!("{update:?}"),
        "RobotSubnetTrafficUpdate([redacted])"
    );
    let subnet_state = decode_get(RobotSubnetGetRequest::new(subnet("192.0.2.10")), DETAIL)
        .unwrap_or_else(|_| unreachable!("subnet fixture failed"));
    assert_eq!(
        format!("{:?}", subnet_state.traffic()),
        "RobotSubnetTrafficPolicy([redacted])"
    );
}
