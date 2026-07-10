//! Load Balancer endpoint and request domains.

mod actions;
mod metrics;
mod public_ip;
mod resources;
mod services;
mod targets;
mod types;

pub use actions::*;
pub use metrics::*;
pub use resources::*;
pub use services::*;
pub use targets::*;
pub use types::*;

use crate::EndpointGroup;

/// Load Balancer endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::LoadBalancers,
    EndpointGroup::LoadBalancerActions,
    EndpointGroup::LoadBalancerTypes,
];

#[cfg(test)]
mod tests;
