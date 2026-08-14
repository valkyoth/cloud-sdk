use alloc::vec::Vec;

use super::super::{RobotOrderChoice, RobotOrderLocation, RobotOrderProductId};
use super::{RobotOrderPrice, RobotOrderText, RobotOrderableAddon};

/// Maximum standard products admitted from one catalog response.
pub const MAX_ROBOT_STANDARD_PRODUCTS: usize = 4_096;

/// One complete standard dedicated-server catalog product.
pub struct RobotStandardProduct {
    pub(in crate::robot::ordering) id: RobotOrderProductId,
    pub(in crate::robot::ordering) name: RobotOrderText,
    pub(in crate::robot::ordering) description: Vec<RobotOrderText>,
    pub(in crate::robot::ordering) traffic: RobotOrderText,
    pub(in crate::robot::ordering) distributions: Vec<RobotOrderChoice>,
    pub(in crate::robot::ordering) languages: Vec<RobotOrderChoice>,
    pub(in crate::robot::ordering) locations: Vec<RobotOrderLocation>,
    pub(in crate::robot::ordering) prices: Vec<RobotOrderPrice>,
    pub(in crate::robot::ordering) addons: Vec<RobotOrderableAddon>,
}

impl RobotStandardProduct {
    /// Returns the reusable product identifier.
    #[must_use]
    pub const fn id(&self) -> &RobotOrderProductId {
        &self.id
    }
    /// Returns the protected provider-owned product name.
    #[must_use]
    pub const fn name(&self) -> &RobotOrderText {
        &self.name
    }
    /// Returns protected description lines in provider order.
    #[must_use]
    pub fn description(&self) -> &[RobotOrderText] {
        &self.description
    }
    /// Returns the protected traffic description.
    #[must_use]
    pub const fn traffic(&self) -> &RobotOrderText {
        &self.traffic
    }
    /// Returns available distributions in provider order.
    #[must_use]
    pub fn distributions(&self) -> &[RobotOrderChoice] {
        &self.distributions
    }
    /// Returns available languages in provider order.
    #[must_use]
    pub fn languages(&self) -> &[RobotOrderChoice] {
        &self.languages
    }
    /// Returns available locations in provider order.
    #[must_use]
    pub fn locations(&self) -> &[RobotOrderLocation] {
        &self.locations
    }
    /// Returns current observed prices in provider order.
    #[must_use]
    pub fn prices(&self) -> &[RobotOrderPrice] {
        &self.prices
    }
    /// Returns current orderable addon definitions.
    #[must_use]
    pub fn orderable_addons(&self) -> &[RobotOrderableAddon] {
        &self.addons
    }
}

impl core::fmt::Debug for RobotStandardProduct {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotStandardProduct([redacted])")
    }
}

/// Bounded standard-server catalog with unique product identifiers.
pub struct RobotStandardProductList(pub(in crate::robot::ordering) Vec<RobotStandardProduct>);

impl RobotStandardProductList {
    /// Returns the complete bounded catalog.
    #[must_use]
    pub fn products(&self) -> &[RobotStandardProduct] {
        &self.0
    }
}

impl core::fmt::Debug for RobotStandardProductList {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotStandardProductList")
            .field("products", &self.0.len())
            .finish()
    }
}
