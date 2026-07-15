//! Prepare a billable Hetzner mutation without executing network I/O.

use cloud_sdk::operation::{OperationImpact, PreparationStorage, PrepareOperation};
use cloud_sdk::transport::StatusCode;
use cloud_sdk_hetzner::cloud::load_balancers::{
    LoadBalancerAlgorithm, LoadBalancerCreateRequest, LoadBalancerName, LoadBalancerType,
};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let name = LoadBalancerName::new("edge")?;
    let load_balancer_type = LoadBalancerType::new("lb11")?;
    let operation = LoadBalancerCreateRequest::new(name, load_balancer_type)
        .with_algorithm(LoadBalancerAlgorithm::LeastConnections)
        .with_public_interface(false);

    let mut target = [0_u8; 128];
    let mut body = [0_u8; 512];
    let prepared = operation.prepare(PreparationStorage::new(&mut target, &mut body))?;

    assert_eq!(
        prepared.transport_request().target().as_str(),
        "/load_balancers"
    );
    assert_eq!(prepared.metadata().impact(), OperationImpact::Mutation);
    assert_eq!(
        prepared.response_policy().success_statuses(),
        &[StatusCode::CREATED]
    );

    // This example deliberately stops before a billable network operation.
    Ok(())
}
