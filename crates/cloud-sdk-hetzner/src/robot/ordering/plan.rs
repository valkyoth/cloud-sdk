use super::RobotOrderCurrency;
use super::model::{
    RobotAddonProduct, RobotMarketProduct, RobotOrderPrice, RobotOrderableAddon,
    RobotStandardProduct,
};
use crate::robot::RobotServerNumber;

/// Failure while binding a non-executable order plan to catalog evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotCatalogPlanError {
    /// A selected catalog index does not exist.
    MissingSelection,
    /// An addon does not belong to the selected product.
    ForeignAddon,
    /// An addon quantity is outside its advertised inclusive bounds.
    InvalidQuantity,
    /// A selected addon price belongs to a different location.
    LocationMismatch,
    /// The same addon was selected more than once.
    DuplicateAddon,
}

impl_static_error!(RobotCatalogPlanError,
    Self::MissingSelection => "Robot order plan selection is missing",
    Self::ForeignAddon => "Robot order plan addon belongs to another product",
    Self::InvalidQuantity => "Robot order plan addon quantity is invalid",
    Self::LocationMismatch => "Robot order plan price locations differ",
    Self::DuplicateAddon => "Robot order plan repeats an addon",
);

/// Mandatory warning carried by every catalog-derived order plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotCatalogPriceWarning {
    /// Catalog prices must be fetched and approved again immediately before purchase.
    RevalidateImmediatelyBeforePurchase,
}

/// One quantity and location-price choice bound to an advertised addon.
pub struct RobotStandardAddonSelection<'a> {
    addon: &'a RobotOrderableAddon,
    price: &'a RobotOrderPrice,
    quantity: u64,
}

impl<'a> RobotStandardAddonSelection<'a> {
    /// Selects one addon price and validates its advertised quantity range.
    pub fn new(
        addon: &'a RobotOrderableAddon,
        price_index: usize,
        quantity: u64,
    ) -> Result<Self, RobotCatalogPlanError> {
        let price = addon
            .prices()
            .get(price_index)
            .ok_or(RobotCatalogPlanError::MissingSelection)?;
        if quantity < addon.minimum() || quantity > addon.maximum() {
            return Err(RobotCatalogPlanError::InvalidQuantity);
        }
        Ok(Self {
            addon,
            price,
            quantity,
        })
    }

    /// Returns the selected addon definition.
    #[must_use]
    pub const fn addon(&self) -> &RobotOrderableAddon {
        self.addon
    }
    /// Returns the selected current price observation.
    #[must_use]
    pub const fn price(&self) -> &RobotOrderPrice {
        self.price
    }
    /// Returns the selected quantity.
    #[must_use]
    pub const fn quantity(&self) -> u64 {
        self.quantity
    }
}

impl core::fmt::Debug for RobotStandardAddonSelection<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotStandardAddonSelection")
            .field("addon", &"[bound]")
            .field("price", &"[bound]")
            .field("quantity", &self.quantity)
            .finish()
    }
}

/// Non-executable standard-server order inputs bound to one catalog snapshot.
pub struct RobotStandardOrderPlan<'a> {
    product: &'a RobotStandardProduct,
    currency: &'a RobotOrderCurrency,
    price: &'a RobotOrderPrice,
    distribution_index: usize,
    language_index: usize,
    addons: &'a [RobotStandardAddonSelection<'a>],
}

impl<'a> RobotStandardOrderPlan<'a> {
    /// Binds selections to one product without creating a purchase request.
    pub fn new(
        product: &'a RobotStandardProduct,
        currency: &'a RobotOrderCurrency,
        price_index: usize,
        distribution_index: usize,
        language_index: usize,
        addons: &'a [RobotStandardAddonSelection<'a>],
    ) -> Result<Self, RobotCatalogPlanError> {
        let price = product
            .prices()
            .get(price_index)
            .ok_or(RobotCatalogPlanError::MissingSelection)?;
        product
            .distributions()
            .get(distribution_index)
            .ok_or(RobotCatalogPlanError::MissingSelection)?;
        product
            .languages()
            .get(language_index)
            .ok_or(RobotCatalogPlanError::MissingSelection)?;
        for (index, selection) in addons.iter().enumerate() {
            if !product
                .orderable_addons()
                .iter()
                .any(|candidate| core::ptr::eq(candidate, selection.addon))
            {
                return Err(RobotCatalogPlanError::ForeignAddon);
            }
            if selection.price.location() != price.location() {
                return Err(RobotCatalogPlanError::LocationMismatch);
            }
            if addons.get(..index).is_some_and(|prior| {
                prior
                    .iter()
                    .any(|candidate| core::ptr::eq(candidate.addon, selection.addon))
            }) {
                return Err(RobotCatalogPlanError::DuplicateAddon);
            }
        }
        Ok(Self {
            product,
            currency,
            price,
            distribution_index,
            language_index,
            addons,
        })
    }

    /// Returns the selected catalog product.
    #[must_use]
    pub const fn product(&self) -> &RobotStandardProduct {
        self.product
    }
    /// Returns the separately observed account currency.
    #[must_use]
    pub const fn currency(&self) -> &RobotOrderCurrency {
        self.currency
    }
    /// Returns the selected current location price.
    #[must_use]
    pub const fn price(&self) -> &RobotOrderPrice {
        self.price
    }
    /// Returns the selected distribution.
    #[must_use]
    pub fn distribution(&self) -> &super::RobotOrderChoice {
        self.product
            .distributions()
            .get(self.distribution_index)
            .unwrap_or_else(|| unreachable!("validated distribution index disappeared"))
    }
    /// Returns the selected language.
    #[must_use]
    pub fn language(&self) -> &super::RobotOrderChoice {
        self.product
            .languages()
            .get(self.language_index)
            .unwrap_or_else(|| unreachable!("validated language index disappeared"))
    }
    /// Returns selected addon quantities and prices.
    #[must_use]
    pub const fn addons(&self) -> &[RobotStandardAddonSelection<'a>] {
        self.addons
    }
    /// Returns the mandatory stale-price warning.
    #[must_use]
    pub const fn price_warning(&self) -> RobotCatalogPriceWarning {
        RobotCatalogPriceWarning::RevalidateImmediatelyBeforePurchase
    }
}

impl core::fmt::Debug for RobotStandardOrderPlan<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotStandardOrderPlan([redacted])")
    }
}

/// Non-executable Server Auction inputs bound to one current catalog item.
pub struct RobotMarketOrderPlan<'a> {
    product: &'a RobotMarketProduct,
    currency: &'a RobotOrderCurrency,
    distribution_index: usize,
    language_index: usize,
}

impl<'a> RobotMarketOrderPlan<'a> {
    /// Binds distribution and language without creating a purchase request.
    pub fn new(
        product: &'a RobotMarketProduct,
        currency: &'a RobotOrderCurrency,
        distribution_index: usize,
        language_index: usize,
    ) -> Result<Self, RobotCatalogPlanError> {
        product
            .distributions()
            .get(distribution_index)
            .ok_or(RobotCatalogPlanError::MissingSelection)?;
        product
            .languages()
            .get(language_index)
            .ok_or(RobotCatalogPlanError::MissingSelection)?;
        Ok(Self {
            product,
            currency,
            distribution_index,
            language_index,
        })
    }

    /// Returns the current Server Auction product observation.
    #[must_use]
    pub const fn product(&self) -> &RobotMarketProduct {
        self.product
    }
    /// Returns the separately observed account currency.
    #[must_use]
    pub const fn currency(&self) -> &RobotOrderCurrency {
        self.currency
    }
    /// Returns the selected distribution.
    #[must_use]
    pub fn distribution(&self) -> &super::RobotOrderChoice {
        self.product
            .distributions()
            .get(self.distribution_index)
            .unwrap_or_else(|| unreachable!("validated market distribution disappeared"))
    }
    /// Returns the selected language.
    #[must_use]
    pub fn language(&self) -> &super::RobotOrderChoice {
        self.product
            .languages()
            .get(self.language_index)
            .unwrap_or_else(|| unreachable!("validated market language disappeared"))
    }
    /// Returns the mandatory stale-price warning.
    #[must_use]
    pub const fn price_warning(&self) -> RobotCatalogPriceWarning {
        RobotCatalogPriceWarning::RevalidateImmediatelyBeforePurchase
    }
}

impl core::fmt::Debug for RobotMarketOrderPlan<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotMarketOrderPlan([redacted])")
    }
}

/// Non-executable per-server addon plan bound to one catalog observation.
pub struct RobotAddonOrderPlan<'a> {
    server: &'a RobotServerNumber,
    product: &'a RobotAddonProduct,
    currency: &'a RobotOrderCurrency,
}

impl<'a> RobotAddonOrderPlan<'a> {
    /// Binds a server and advertised addon without creating a purchase request.
    #[must_use]
    pub const fn new(
        server: &'a RobotServerNumber,
        product: &'a RobotAddonProduct,
        currency: &'a RobotOrderCurrency,
    ) -> Self {
        Self {
            server,
            product,
            currency,
        }
    }
    /// Returns the request-bound server identity.
    #[must_use]
    pub const fn server(&self) -> &RobotServerNumber {
        self.server
    }
    /// Returns the selected advertised addon.
    #[must_use]
    pub const fn product(&self) -> &RobotAddonProduct {
        self.product
    }
    /// Returns the separately observed account currency.
    #[must_use]
    pub const fn currency(&self) -> &RobotOrderCurrency {
        self.currency
    }
    /// Returns the mandatory stale-price warning.
    #[must_use]
    pub const fn price_warning(&self) -> RobotCatalogPriceWarning {
        RobotCatalogPriceWarning::RevalidateImmediatelyBeforePurchase
    }
}

impl core::fmt::Debug for RobotAddonOrderPlan<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotAddonOrderPlan([redacted])")
    }
}
