use super::super::{RobotOrderDecimal, RobotOrderLocation, RobotOrderProductId};
use super::RobotOrderText;

/// Exact net and gross values for one billing dimension.
pub struct RobotOrderPricePair {
    pub(in crate::robot::ordering) net: RobotOrderDecimal,
    pub(in crate::robot::ordering) gross: RobotOrderDecimal,
}

impl RobotOrderPricePair {
    /// Returns the exact net amount.
    #[must_use]
    pub const fn net(&self) -> &RobotOrderDecimal {
        &self.net
    }

    /// Returns the exact gross amount.
    #[must_use]
    pub const fn gross(&self) -> &RobotOrderDecimal {
        &self.gross
    }
}

impl core::fmt::Debug for RobotOrderPricePair {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotOrderPricePair([redacted])")
    }
}

/// Exact location-specific recurring and setup prices.
pub struct RobotOrderPrice {
    pub(in crate::robot::ordering) location: RobotOrderLocation,
    pub(in crate::robot::ordering) recurring: RobotOrderPricePair,
    pub(in crate::robot::ordering) hourly: Option<RobotOrderPricePair>,
    pub(in crate::robot::ordering) setup: RobotOrderPricePair,
}

impl RobotOrderPrice {
    /// Returns the location to which this price applies.
    #[must_use]
    pub const fn location(&self) -> &RobotOrderLocation {
        &self.location
    }

    /// Returns monthly net and gross values.
    #[must_use]
    pub const fn recurring(&self) -> &RobotOrderPricePair {
        &self.recurring
    }

    /// Returns hourly net and gross values when hourly billing is offered.
    #[must_use]
    pub const fn hourly(&self) -> Option<&RobotOrderPricePair> {
        self.hourly.as_ref()
    }

    /// Returns one-time setup net and gross values.
    #[must_use]
    pub const fn setup(&self) -> &RobotOrderPricePair {
        &self.setup
    }
}

impl core::fmt::Debug for RobotOrderPrice {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotOrderPrice([redacted])")
    }
}

/// One bounded addon advertised with a standard or Server Auction product.
pub struct RobotOrderableAddon {
    pub(in crate::robot::ordering) id: RobotOrderProductId,
    pub(in crate::robot::ordering) name: RobotOrderText,
    pub(in crate::robot::ordering) minimum: u64,
    pub(in crate::robot::ordering) maximum: u64,
    pub(in crate::robot::ordering) prices: alloc::vec::Vec<RobotOrderPrice>,
}

impl RobotOrderableAddon {
    /// Returns the reusable addon identifier.
    #[must_use]
    pub const fn id(&self) -> &RobotOrderProductId {
        &self.id
    }

    /// Returns the protected provider-owned addon name.
    #[must_use]
    pub const fn name(&self) -> &RobotOrderText {
        &self.name
    }

    /// Returns the minimum orderable quantity.
    #[must_use]
    pub const fn minimum(&self) -> u64 {
        self.minimum
    }

    /// Returns the maximum orderable quantity.
    #[must_use]
    pub const fn maximum(&self) -> u64 {
        self.maximum
    }

    /// Returns location-specific observed prices.
    #[must_use]
    pub fn prices(&self) -> &[RobotOrderPrice] {
        &self.prices
    }
}

impl core::fmt::Debug for RobotOrderableAddon {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotOrderableAddon([redacted])")
    }
}
