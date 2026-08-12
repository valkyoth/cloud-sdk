use alloc::format;

use cloud_sdk::operation::PermitTimestamp;

use super::tests::{MAC_SUBNET_DETAIL, decode_get, decode_mac_get, subnet, text};
use super::{
    RobotSubnetGetRequest, RobotSubnetMacDeleteRequest, RobotSubnetMacGetRequest,
    RobotSubnetMutationLease, RobotSubnetObservationWindow,
};

pub(super) fn delete_request() -> RobotSubnetMacDeleteRequest {
    delete_request_with(
        "192.0.2.1",
        "00:21:85:62:3e:9c",
        b"test-lock-generation-0001",
        99,
        100,
        130,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn delete_request_with(
    server: &str,
    default_mac: &str,
    lock_id: &[u8],
    subnet_observed_at: u64,
    mac_observed_at: u64,
    lease_expires_at: u64,
) -> RobotSubnetMacDeleteRequest {
    let subnet_body = text(MAC_SUBNET_DETAIL).replace("192.0.2.1", server);
    let subnet_state = decode_get(
        RobotSubnetGetRequest::new(subnet("2001:db8::")),
        subnet_body.as_bytes(),
    )
    .unwrap_or_else(|_| unreachable!("subnet evidence failed"));
    let selectable = if default_mac == "00:21:85:62:3e:9d" {
        format!(r#""{server}":"{default_mac}""#)
    } else {
        format!(r#""{server}":"{default_mac}","192.0.2.254":"00:21:85:62:3e:9d""#)
    };
    let mac_body = format!(
        r#"{{"mac":{{"ip":"2001:db8::","mask":"64","mac":"00:21:85:62:3e:9d","possible_mac":{{{selectable}}}}}}}"#
    );
    let mac_state = decode_mac_get(
        RobotSubnetMacGetRequest::new(subnet("2001:db8::")),
        mac_body.as_bytes(),
    )
    .unwrap_or_else(|_| unreachable!("MAC evidence failed"));
    let observations = RobotSubnetObservationWindow::new(
        PermitTimestamp::from_seconds(subnet_observed_at),
        PermitTimestamp::from_seconds(mac_observed_at),
    )
    .unwrap_or_else(|_| unreachable!("observation window failed"));
    let lease = RobotSubnetMutationLease::new(
        subnet("2001:db8::"),
        lock_id,
        PermitTimestamp::from_seconds(lease_expires_at),
    )
    .unwrap_or_else(|_| unreachable!("mutation lease failed"));
    RobotSubnetMacDeleteRequest::from_checked(subnet_state, mac_state, observations, lease)
        .unwrap_or_else(|_| unreachable!("default MAC evidence failed"))
}

pub(super) fn observations() -> RobotSubnetObservationWindow {
    RobotSubnetObservationWindow::new(
        PermitTimestamp::from_seconds(99),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("observation window failed"))
}

pub(super) fn mutation_lease() -> RobotSubnetMutationLease {
    RobotSubnetMutationLease::new(
        subnet("2001:db8::"),
        b"test-lock-generation-0001",
        PermitTimestamp::from_seconds(130),
    )
    .unwrap_or_else(|_| unreachable!("mutation lease failed"))
}
