use alloc::vec::Vec;

use super::super::{RobotMarketProductId, RobotOrderChoice, RobotOrderDecimal};
use super::{RobotOrderText, RobotOrderableAddon};

/// Maximum Server Auction products admitted from one catalog response.
pub const MAX_ROBOT_MARKET_PRODUCTS: usize = 4_096;

/// One complete Server Auction catalog product.
pub struct RobotMarketProduct {
    pub(in crate::robot::ordering) id: RobotMarketProductId,
    pub(in crate::robot::ordering) name: RobotOrderText,
    pub(in crate::robot::ordering) description: Vec<RobotOrderText>,
    pub(in crate::robot::ordering) traffic: RobotOrderText,
    pub(in crate::robot::ordering) distributions: Vec<RobotOrderChoice>,
    pub(in crate::robot::ordering) languages: Vec<RobotOrderChoice>,
    pub(in crate::robot::ordering) cpu: RobotOrderText,
    pub(in crate::robot::ordering) cpu_benchmark: u64,
    pub(in crate::robot::ordering) memory_size: u64,
    pub(in crate::robot::ordering) hdd_size: u64,
    pub(in crate::robot::ordering) hdd_text: RobotOrderText,
    pub(in crate::robot::ordering) hdd_count: u64,
    pub(in crate::robot::ordering) datacenter: RobotOrderText,
    pub(in crate::robot::ordering) network_speed: RobotOrderText,
    pub(in crate::robot::ordering) monthly_net: RobotOrderDecimal,
    pub(in crate::robot::ordering) hourly_net: Option<RobotOrderDecimal>,
    pub(in crate::robot::ordering) setup_net: RobotOrderDecimal,
    pub(in crate::robot::ordering) monthly_gross: RobotOrderDecimal,
    pub(in crate::robot::ordering) hourly_gross: Option<RobotOrderDecimal>,
    pub(in crate::robot::ordering) setup_gross: RobotOrderDecimal,
    pub(in crate::robot::ordering) fixed_price: bool,
    pub(in crate::robot::ordering) next_reduce_seconds: i64,
    pub(in crate::robot::ordering) next_reduce_at: RobotOrderText,
    pub(in crate::robot::ordering) addons: Vec<RobotOrderableAddon>,
}

impl RobotMarketProduct {
    /// Returns the Server Auction product identifier.
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
    /// Returns available distributions.
    #[must_use]
    pub fn distributions(&self) -> &[RobotOrderChoice] {
        &self.distributions
    }
    /// Returns available languages.
    #[must_use]
    pub fn languages(&self) -> &[RobotOrderChoice] {
        &self.languages
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
    /// Returns memory size in GiB as supplied by Robot.
    #[must_use]
    pub const fn memory_size(&self) -> u64 {
        self.memory_size
    }
    /// Returns drive size in GiB as supplied by Robot.
    #[must_use]
    pub const fn hdd_size(&self) -> u64 {
        self.hdd_size
    }
    /// Returns the protected drive description.
    #[must_use]
    pub const fn hdd_text(&self) -> &RobotOrderText {
        &self.hdd_text
    }
    /// Returns the drive count.
    #[must_use]
    pub const fn hdd_count(&self) -> u64 {
        self.hdd_count
    }
    /// Returns the protected datacenter identifier.
    #[must_use]
    pub const fn datacenter(&self) -> &RobotOrderText {
        &self.datacenter
    }
    /// Returns the protected network-speed description.
    #[must_use]
    pub const fn network_speed(&self) -> &RobotOrderText {
        &self.network_speed
    }
    /// Returns the current observed monthly net price.
    #[must_use]
    pub const fn monthly_net(&self) -> &RobotOrderDecimal {
        &self.monthly_net
    }
    /// Returns the current observed hourly net price.
    #[must_use]
    pub const fn hourly_net(&self) -> Option<&RobotOrderDecimal> {
        self.hourly_net.as_ref()
    }
    /// Returns the current observed one-time net setup price.
    #[must_use]
    pub const fn setup_net(&self) -> &RobotOrderDecimal {
        &self.setup_net
    }
    /// Returns the current observed monthly gross price.
    #[must_use]
    pub const fn monthly_gross(&self) -> &RobotOrderDecimal {
        &self.monthly_gross
    }
    /// Returns the current observed hourly gross price.
    #[must_use]
    pub const fn hourly_gross(&self) -> Option<&RobotOrderDecimal> {
        self.hourly_gross.as_ref()
    }
    /// Returns the current observed one-time gross setup price.
    #[must_use]
    pub const fn setup_gross(&self) -> &RobotOrderDecimal {
        &self.setup_gross
    }
    /// Reports whether the auction currently has a fixed price.
    #[must_use]
    pub const fn fixed_price(&self) -> bool {
        self.fixed_price
    }
    /// Returns seconds until the next reduction; negative values are retained.
    #[must_use]
    pub const fn next_reduce_seconds(&self) -> i64 {
        self.next_reduce_seconds
    }
    /// Returns the provider's protected next-reduction timestamp text.
    #[must_use]
    pub const fn next_reduce_at(&self) -> &RobotOrderText {
        &self.next_reduce_at
    }
    /// Returns current orderable addon definitions.
    #[must_use]
    pub fn orderable_addons(&self) -> &[RobotOrderableAddon] {
        &self.addons
    }
}

impl core::fmt::Debug for RobotMarketProduct {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotMarketProduct([redacted])")
    }
}

/// Bounded Server Auction catalog with unique product identifiers.
pub struct RobotMarketProductList(pub(in crate::robot::ordering) Vec<RobotMarketProduct>);

impl RobotMarketProductList {
    /// Returns the complete bounded catalog.
    #[must_use]
    pub fn products(&self) -> &[RobotMarketProduct] {
        &self.0
    }
}

impl core::fmt::Debug for RobotMarketProductList {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotMarketProductList")
            .field("products", &self.0.len())
            .finish()
    }
}
