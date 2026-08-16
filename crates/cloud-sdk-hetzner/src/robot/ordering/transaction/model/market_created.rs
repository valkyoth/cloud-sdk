use alloc::vec::Vec;

use crate::robot::ordering::{
    RobotMarketProductId, RobotOrderChoice, RobotOrderProductId, RobotOrderText,
    RobotOrderTransactionId,
};
use crate::robot::{ProtectedIpAddr, RobotServerNumber};

use super::common::{
    RobotOrderTransactionKey, RobotOrderTransactionStatus, RobotOrderTransactionTimestamp,
};

/// Product shape returned specifically by Server Auction creation.
pub struct RobotMarketCreatedProduct {
    pub(in crate::robot::ordering) id: RobotMarketProductId,
    pub(in crate::robot::ordering) name: RobotOrderText,
    pub(in crate::robot::ordering) description: Vec<RobotOrderText>,
    pub(in crate::robot::ordering) traffic: RobotOrderText,
    pub(in crate::robot::ordering) distribution: RobotOrderChoice,
    pub(in crate::robot::ordering) architecture: u8,
    pub(in crate::robot::ordering) language: RobotOrderChoice,
    pub(in crate::robot::ordering) cpu: RobotOrderText,
    pub(in crate::robot::ordering) cpu_benchmark: u64,
    pub(in crate::robot::ordering) memory_size: u64,
    pub(in crate::robot::ordering) hdd_size: u64,
    pub(in crate::robot::ordering) hdd_text: RobotOrderText,
    pub(in crate::robot::ordering) hdd_count: u64,
    pub(in crate::robot::ordering) datacenter: RobotOrderText,
    pub(in crate::robot::ordering) network_speed: RobotOrderText,
}

impl RobotMarketCreatedProduct {
    /// Returns the ordered auction product identifier.
    #[must_use]
    pub const fn id(&self) -> RobotMarketProductId {
        self.id
    }
    /// Returns the protected product name.
    #[must_use]
    pub const fn name(&self) -> &RobotOrderText {
        &self.name
    }
    /// Returns protected product description lines.
    #[must_use]
    pub fn description(&self) -> &[RobotOrderText] {
        &self.description
    }
    /// Returns the protected traffic description.
    #[must_use]
    pub const fn traffic(&self) -> &RobotOrderText {
        &self.traffic
    }
    /// Returns the ordered distribution.
    #[must_use]
    pub const fn distribution(&self) -> &RobotOrderChoice {
        &self.distribution
    }
    /// Returns the deprecated source architecture value.
    #[must_use]
    pub const fn architecture(&self) -> u8 {
        self.architecture
    }
    /// Returns the ordered language.
    #[must_use]
    pub const fn language(&self) -> &RobotOrderChoice {
        &self.language
    }
    /// Returns the protected CPU description.
    #[must_use]
    pub const fn cpu(&self) -> &RobotOrderText {
        &self.cpu
    }
    /// Returns the provider CPU benchmark.
    #[must_use]
    pub const fn cpu_benchmark(&self) -> u64 {
        self.cpu_benchmark
    }
    /// Returns memory size in GiB.
    #[must_use]
    pub const fn memory_size(&self) -> u64 {
        self.memory_size
    }
    /// Returns drive size in GiB.
    #[must_use]
    pub const fn hdd_size(&self) -> u64 {
        self.hdd_size
    }
    /// Returns the protected drive description.
    #[must_use]
    pub const fn hdd_text(&self) -> &RobotOrderText {
        &self.hdd_text
    }
    /// Returns drive count.
    #[must_use]
    pub const fn hdd_count(&self) -> u64 {
        self.hdd_count
    }
    /// Returns the protected datacenter.
    #[must_use]
    pub const fn datacenter(&self) -> &RobotOrderText {
        &self.datacenter
    }
    /// Returns the protected network speed.
    #[must_use]
    pub const fn network_speed(&self) -> &RobotOrderText {
        &self.network_speed
    }
}

impl core::fmt::Debug for RobotMarketCreatedProduct {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotMarketCreatedProduct([redacted])")
    }
}

/// Successful Server Auction creation response.
pub struct RobotMarketCreatedTransaction {
    pub(in crate::robot::ordering) id: RobotOrderTransactionId,
    pub(in crate::robot::ordering) date: RobotOrderTransactionTimestamp,
    pub(in crate::robot::ordering) status: RobotOrderTransactionStatus,
    pub(in crate::robot::ordering) server_number: Option<RobotServerNumber>,
    pub(in crate::robot::ordering) server_ip: Option<ProtectedIpAddr>,
    pub(in crate::robot::ordering) authorized_keys: Vec<RobotOrderTransactionKey>,
    pub(in crate::robot::ordering) host_keys: Vec<RobotOrderTransactionKey>,
    pub(in crate::robot::ordering) comment: Option<RobotOrderText>,
    pub(in crate::robot::ordering) product: RobotMarketCreatedProduct,
    pub(in crate::robot::ordering) addons: Vec<RobotOrderProductId>,
}

impl RobotMarketCreatedTransaction {
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
    /// Returns the resulting server number only when ready.
    #[must_use]
    pub const fn server_number(&self) -> Option<&RobotServerNumber> {
        self.server_number.as_ref()
    }
    /// Returns the resulting server address only when ready.
    #[must_use]
    pub const fn server_ip(&self) -> Option<&ProtectedIpAddr> {
        self.server_ip.as_ref()
    }
    /// Returns supplied authorized-key metadata.
    #[must_use]
    pub fn authorized_keys(&self) -> &[RobotOrderTransactionKey] {
        &self.authorized_keys
    }
    /// Returns observed server host-key metadata.
    #[must_use]
    pub fn host_keys(&self) -> &[RobotOrderTransactionKey] {
        &self.host_keys
    }
    /// Returns the protected order comment when supplied.
    #[must_use]
    pub const fn comment(&self) -> Option<&RobotOrderText> {
        self.comment.as_ref()
    }
    /// Returns the creation-specific auction product snapshot.
    #[must_use]
    pub const fn product(&self) -> &RobotMarketCreatedProduct {
        &self.product
    }
    /// Returns addon identifiers in the creation response.
    #[must_use]
    pub fn addons(&self) -> &[RobotOrderProductId] {
        &self.addons
    }
}

impl core::fmt::Debug for RobotMarketCreatedTransaction {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotMarketCreatedTransaction([redacted])")
    }
}
