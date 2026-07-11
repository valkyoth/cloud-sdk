//! DNS Zone endpoint and request domains.

mod actions;
mod resources;
mod types;

pub use actions::*;
pub use resources::*;
pub use types::*;

use crate::EndpointGroup;

/// Zone endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[EndpointGroup::Zones, EndpointGroup::ZoneActions];

#[cfg(test)]
mod tests;
