use alloc::vec::Vec;

use crate::robot::RobotServerNumber;
use crate::robot::ordering::{
    RobotOrderPrice, RobotOrderProductId, RobotOrderText, RobotOrderTransactionId,
};

use super::common::{
    RobotOrderTransactionResource, RobotOrderTransactionStatus, RobotOrderTransactionTimestamp,
};

/// Addon product and exact observed price retained by one transaction.
pub struct RobotAddonTransactionProduct {
    pub(in crate::robot::ordering) id: RobotOrderProductId,
    pub(in crate::robot::ordering) name: RobotOrderText,
    pub(in crate::robot::ordering) price: RobotOrderPrice,
}

impl RobotAddonTransactionProduct {
    /// Returns the ordered addon product identifier.
    #[must_use]
    pub const fn id(&self) -> &RobotOrderProductId {
        &self.id
    }
    /// Returns the protected addon name.
    #[must_use]
    pub const fn name(&self) -> &RobotOrderText {
        &self.name
    }
    /// Returns the exact location-specific price retained by the transaction.
    #[must_use]
    pub const fn price(&self) -> &RobotOrderPrice {
        &self.price
    }
}

impl core::fmt::Debug for RobotAddonTransactionProduct {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotAddonTransactionProduct([redacted])")
    }
}

/// Complete per-server addon transaction.
pub struct RobotAddonTransaction {
    pub(in crate::robot::ordering) id: RobotOrderTransactionId,
    pub(in crate::robot::ordering) date: RobotOrderTransactionTimestamp,
    pub(in crate::robot::ordering) status: RobotOrderTransactionStatus,
    pub(in crate::robot::ordering) server_number: RobotServerNumber,
    pub(in crate::robot::ordering) product: RobotAddonTransactionProduct,
    pub(in crate::robot::ordering) resources: Vec<RobotOrderTransactionResource>,
}

impl RobotAddonTransaction {
    /// Returns the protected transaction identifier.
    #[must_use]
    pub const fn id(&self) -> &RobotOrderTransactionId {
        &self.id
    }
    /// Returns the protected source timestamp.
    #[must_use]
    pub const fn date(&self) -> &RobotOrderTransactionTimestamp {
        &self.date
    }
    /// Returns the finite transaction state.
    #[must_use]
    pub const fn status(&self) -> RobotOrderTransactionStatus {
        self.status
    }
    /// Returns the server to which the addon order belongs.
    #[must_use]
    pub const fn server_number(&self) -> &RobotServerNumber {
        &self.server_number
    }
    /// Returns the ordered addon product snapshot.
    #[must_use]
    pub const fn product(&self) -> &RobotAddonTransactionProduct {
        &self.product
    }
    /// Returns resources created by the order.
    #[must_use]
    pub fn resources(&self) -> &[RobotOrderTransactionResource] {
        &self.resources
    }
}

impl core::fmt::Debug for RobotAddonTransaction {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotAddonTransaction([redacted])")
    }
}

/// Bounded addon transaction snapshot with unique identifiers.
pub struct RobotAddonTransactionList(pub(in crate::robot::ordering) Vec<RobotAddonTransaction>);
impl RobotAddonTransactionList {
    /// Returns transactions from Robot's fixed 30-day window.
    #[must_use]
    pub fn transactions(&self) -> &[RobotAddonTransaction] {
        &self.0
    }
}
impl core::fmt::Debug for RobotAddonTransactionList {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotAddonTransactionList")
            .field("transactions", &self.0.len())
            .finish()
    }
}
