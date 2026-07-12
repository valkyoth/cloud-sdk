//! DNS RRSet endpoint and request domains.

mod actions;
mod path;
mod resources;
mod types;

pub use actions::*;
pub use resources::*;
pub use types::*;

use crate::EndpointGroup;

/// RRSet endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] =
    &[EndpointGroup::ZoneRrsets, EndpointGroup::ZoneRrsetActions];

#[cfg(test)]
mod tests;
