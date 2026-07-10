//! Network resource requests, routes, and subnets.

mod requests;
mod types;

pub use requests::{
    NetworkCreateRequest, NetworkEndpoint, NetworkListRequest, NetworkUpdateRequest,
};
pub use types::{
    NetworkId, NetworkLabels, NetworkName, NetworkRequestError, NetworkRoute, NetworkSortField,
    NetworkSubnet, NetworkSubnetType, NetworkVswitchId, NetworkZone,
};

#[cfg(test)]
mod tests;
