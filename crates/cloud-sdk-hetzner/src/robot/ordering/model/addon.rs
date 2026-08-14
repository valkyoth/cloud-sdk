use alloc::vec::Vec;

use super::super::RobotOrderProductId;
use super::{RobotOrderPrice, RobotOrderText};

/// Maximum per-server addon products admitted from one response.
pub const MAX_ROBOT_ADDON_PRODUCTS: usize = 4_096;

/// One addon currently available for a specific server.
pub struct RobotAddonProduct {
    pub(in crate::robot::ordering) id: RobotOrderProductId,
    pub(in crate::robot::ordering) name: RobotOrderText,
    pub(in crate::robot::ordering) kind: RobotOrderText,
    pub(in crate::robot::ordering) price: RobotOrderPrice,
}

impl RobotAddonProduct {
    /// Returns the reusable addon product identifier.
    #[must_use]
    pub const fn id(&self) -> &RobotOrderProductId {
        &self.id
    }
    /// Returns the protected provider-owned addon name.
    #[must_use]
    pub const fn name(&self) -> &RobotOrderText {
        &self.name
    }
    /// Returns the protected provider-owned addon type.
    #[must_use]
    pub const fn kind(&self) -> &RobotOrderText {
        &self.kind
    }
    /// Returns the current observed location-specific price.
    #[must_use]
    pub const fn price(&self) -> &RobotOrderPrice {
        &self.price
    }
}

impl core::fmt::Debug for RobotAddonProduct {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotAddonProduct([redacted])")
    }
}

/// Bounded per-server addon catalog with unique product identifiers.
pub struct RobotAddonProductList(pub(in crate::robot::ordering) Vec<RobotAddonProduct>);

impl RobotAddonProductList {
    /// Returns all currently available addons.
    #[must_use]
    pub fn products(&self) -> &[RobotAddonProduct] {
        &self.0
    }
}

impl core::fmt::Debug for RobotAddonProductList {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotAddonProductList")
            .field("products", &self.0.len())
            .finish()
    }
}
