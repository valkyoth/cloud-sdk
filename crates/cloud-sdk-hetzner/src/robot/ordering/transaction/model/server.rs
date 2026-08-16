use alloc::vec::Vec;

use crate::robot::ordering::{
    RobotMarketProductId, RobotOrderChoice, RobotOrderLocation, RobotOrderProductId,
    RobotOrderText, RobotOrderTransactionId,
};
use crate::robot::{ProtectedIpAddr, RobotServerNumber};

use super::common::{
    RobotOrderTransactionKey, RobotOrderTransactionStatus, RobotOrderTransactionTimestamp,
};

/// Product snapshot retained by a standard-server transaction.
pub struct RobotStandardTransactionProduct {
    pub(in crate::robot::ordering) id: RobotOrderProductId,
    pub(in crate::robot::ordering) name: RobotOrderText,
    pub(in crate::robot::ordering) description: Vec<RobotOrderText>,
    pub(in crate::robot::ordering) traffic: RobotOrderText,
    pub(in crate::robot::ordering) distribution: RobotOrderChoice,
    pub(in crate::robot::ordering) architecture: u8,
    pub(in crate::robot::ordering) language: RobotOrderChoice,
    pub(in crate::robot::ordering) location: Option<RobotOrderLocation>,
}

impl RobotStandardTransactionProduct {
    /// Returns the ordered product identifier.
    #[must_use]
    pub const fn id(&self) -> &RobotOrderProductId {
        &self.id
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
    /// Returns the source architecture value, either 32 or 64.
    #[must_use]
    pub const fn architecture(&self) -> u8 {
        self.architecture
    }
    /// Returns the ordered language.
    #[must_use]
    pub const fn language(&self) -> &RobotOrderChoice {
        &self.language
    }
    /// Returns the ordered location when Robot supplied one.
    #[must_use]
    pub const fn location(&self) -> Option<&RobotOrderLocation> {
        self.location.as_ref()
    }
}

impl core::fmt::Debug for RobotStandardTransactionProduct {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotStandardTransactionProduct([redacted])")
    }
}

/// Complete standard-server order transaction.
pub struct RobotStandardTransaction {
    pub(in crate::robot::ordering) id: RobotOrderTransactionId,
    pub(in crate::robot::ordering) date: RobotOrderTransactionTimestamp,
    pub(in crate::robot::ordering) status: RobotOrderTransactionStatus,
    pub(in crate::robot::ordering) server_number: Option<RobotServerNumber>,
    pub(in crate::robot::ordering) server_ip: Option<ProtectedIpAddr>,
    pub(in crate::robot::ordering) authorized_keys: Vec<RobotOrderTransactionKey>,
    pub(in crate::robot::ordering) host_keys: Vec<RobotOrderTransactionKey>,
    pub(in crate::robot::ordering) comment: Option<RobotOrderText>,
    pub(in crate::robot::ordering) product: RobotStandardTransactionProduct,
    pub(in crate::robot::ordering) addons: Vec<RobotOrderProductId>,
}

macro_rules! common_accessors {
    () => {
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
        /// Returns the resulting server number only for a ready transaction.
        #[must_use]
        pub const fn server_number(&self) -> Option<&RobotServerNumber> {
            self.server_number.as_ref()
        }
        /// Returns the resulting server address only for a ready transaction.
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
    };
}

impl RobotStandardTransaction {
    common_accessors!();
    /// Returns the ordered standard-server product snapshot.
    #[must_use]
    pub const fn product(&self) -> &RobotStandardTransactionProduct {
        &self.product
    }
    /// Returns ordered addon identifiers.
    #[must_use]
    pub fn addons(&self) -> &[RobotOrderProductId] {
        &self.addons
    }
}

impl core::fmt::Debug for RobotStandardTransaction {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotStandardTransaction([redacted])")
    }
}

/// Bounded standard-server transaction snapshot with unique identifiers.
pub struct RobotStandardTransactionList(
    pub(in crate::robot::ordering) Vec<RobotStandardTransaction>,
);
impl RobotStandardTransactionList {
    /// Returns transactions from Robot's fixed 30-day window.
    #[must_use]
    pub fn transactions(&self) -> &[RobotStandardTransaction] {
        &self.0
    }
}
impl core::fmt::Debug for RobotStandardTransactionList {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotStandardTransactionList")
            .field("transactions", &self.0.len())
            .finish()
    }
}

/// Product snapshot retained by a Server Auction transaction.
pub struct RobotMarketTransactionProduct {
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
    pub(in crate::robot::ordering) fixed_price: bool,
    pub(in crate::robot::ordering) next_reduce_seconds: i64,
    pub(in crate::robot::ordering) next_reduce_at: RobotOrderText,
}

impl RobotMarketTransactionProduct {
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
    /// Returns protected description lines.
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
    /// Returns the source architecture value, either 32 or 64.
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
    /// Reports whether the observed auction price was fixed.
    #[must_use]
    pub const fn fixed_price(&self) -> bool {
        self.fixed_price
    }
    /// Returns seconds until the next reduction as observed at order time.
    #[must_use]
    pub const fn next_reduce_seconds(&self) -> i64 {
        self.next_reduce_seconds
    }
    /// Returns the protected next-reduction timestamp text.
    #[must_use]
    pub const fn next_reduce_at(&self) -> &RobotOrderText {
        &self.next_reduce_at
    }
}

impl core::fmt::Debug for RobotMarketTransactionProduct {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotMarketTransactionProduct([redacted])")
    }
}

/// Complete Server Auction order transaction.
pub struct RobotMarketTransaction {
    pub(in crate::robot::ordering) id: RobotOrderTransactionId,
    pub(in crate::robot::ordering) date: RobotOrderTransactionTimestamp,
    pub(in crate::robot::ordering) status: RobotOrderTransactionStatus,
    pub(in crate::robot::ordering) server_number: Option<RobotServerNumber>,
    pub(in crate::robot::ordering) server_ip: Option<ProtectedIpAddr>,
    pub(in crate::robot::ordering) authorized_keys: Vec<RobotOrderTransactionKey>,
    pub(in crate::robot::ordering) host_keys: Vec<RobotOrderTransactionKey>,
    pub(in crate::robot::ordering) comment: Option<RobotOrderText>,
    pub(in crate::robot::ordering) product: RobotMarketTransactionProduct,
}

impl RobotMarketTransaction {
    common_accessors!();
    /// Returns the ordered Server Auction product snapshot.
    #[must_use]
    pub const fn product(&self) -> &RobotMarketTransactionProduct {
        &self.product
    }
}
impl core::fmt::Debug for RobotMarketTransaction {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotMarketTransaction([redacted])")
    }
}

/// Bounded Server Auction transaction snapshot with unique identifiers.
pub struct RobotMarketTransactionList(pub(in crate::robot::ordering) Vec<RobotMarketTransaction>);
impl RobotMarketTransactionList {
    /// Returns transactions from Robot's fixed 30-day window.
    #[must_use]
    pub fn transactions(&self) -> &[RobotMarketTransaction] {
        &self.0
    }
}
impl core::fmt::Debug for RobotMarketTransactionList {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotMarketTransactionList")
            .field("transactions", &self.0.len())
            .finish()
    }
}
