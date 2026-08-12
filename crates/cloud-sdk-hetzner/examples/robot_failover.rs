//! Prepares one source-locked Robot failover read without network access.

#[cfg(feature = "serde")]
fn main() -> Result<(), Box<dyn core::error::Error>> {
    use cloud_sdk::operation::PreparationStorage;
    use cloud_sdk_hetzner::robot::{RobotFailoverGetRequest, RobotIpAddress};

    let route = RobotIpAddress::new("192.0.2.50")?;
    let request = RobotFailoverGetRequest::new(route);
    let mut target = [0_u8; 64];
    let mut body = [0_u8; 1];
    let prepared = request.prepare_bound(PreparationStorage::new(&mut target, &mut body))?;

    assert_eq!(
        prepared.as_untyped().transport_request().target().as_str(),
        "/failover/192.0.2.50",
    );
    Ok(())
}

#[cfg(not(feature = "serde"))]
fn main() {}
