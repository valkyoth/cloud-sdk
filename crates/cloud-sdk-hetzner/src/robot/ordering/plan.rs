use super::CredentialObserved;
use super::RobotOrderCurrency;
use super::exchange::RobotAddonCatalog;
use super::model::{
    RobotAddonProduct, RobotMarketProduct, RobotOrderPrice, RobotOrderableAddon,
    RobotStandardProduct,
};
use crate::robot::RobotServerNumber;
use cloud_sdk::authentication::CredentialBinding;

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
    /// Catalog and currency observations came from different credential lifecycles.
    CredentialMismatch,
}

impl_static_error!(RobotCatalogPlanError,
    Self::MissingSelection => "Robot order plan selection is missing",
    Self::ForeignAddon => "Robot order plan addon belongs to another product",
    Self::InvalidQuantity => "Robot order plan addon quantity is invalid",
    Self::LocationMismatch => "Robot order plan price locations differ",
    Self::DuplicateAddon => "Robot order plan repeats an addon",
    Self::CredentialMismatch => "Robot order plan observations use different credentials",
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
        formatter.write_str("RobotStandardAddonSelection([redacted])")
    }
}

/// Non-executable standard-server order inputs bound to one catalog snapshot.
pub struct RobotStandardOrderPlan<'a> {
    product: &'a RobotStandardProduct,
    currency: &'a RobotOrderCurrency,
    price: &'a RobotOrderPrice,
    distribution: &'a super::RobotOrderChoice,
    language: &'a super::RobotOrderChoice,
    addons: &'a [RobotStandardAddonSelection<'a>],
    credential: CredentialBinding,
}

impl<'a> RobotStandardOrderPlan<'a> {
    /// Binds selections to one product without creating a purchase request.
    pub fn new(
        product: &'a CredentialObserved<RobotStandardProduct>,
        currency: &'a CredentialObserved<RobotOrderCurrency>,
        price_index: usize,
        distribution_index: usize,
        language_index: usize,
        addons: &'a [RobotStandardAddonSelection<'a>],
    ) -> Result<Self, RobotCatalogPlanError> {
        if !product.credential().matches(currency.credential()) {
            return Err(RobotCatalogPlanError::CredentialMismatch);
        }
        let credential = product.credential();
        let product = product.value();
        let currency = currency.value();
        let price = product
            .prices()
            .get(price_index)
            .ok_or(RobotCatalogPlanError::MissingSelection)?;
        let distribution = product
            .distributions()
            .get(distribution_index)
            .ok_or(RobotCatalogPlanError::MissingSelection)?;
        let language = product
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
            distribution,
            language,
            addons,
            credential,
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
    pub const fn distribution(&self) -> &super::RobotOrderChoice {
        self.distribution
    }
    /// Returns the selected language.
    #[must_use]
    pub const fn language(&self) -> &super::RobotOrderChoice {
        self.language
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

    pub(super) const fn credential(&self) -> CredentialBinding {
        self.credential
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
    distribution: &'a super::RobotOrderChoice,
    language: &'a super::RobotOrderChoice,
    credential: CredentialBinding,
}

impl<'a> RobotMarketOrderPlan<'a> {
    /// Binds distribution and language without creating a purchase request.
    pub fn new(
        product: &'a CredentialObserved<RobotMarketProduct>,
        currency: &'a CredentialObserved<RobotOrderCurrency>,
        distribution_index: usize,
        language_index: usize,
    ) -> Result<Self, RobotCatalogPlanError> {
        if !product.credential().matches(currency.credential()) {
            return Err(RobotCatalogPlanError::CredentialMismatch);
        }
        let credential = product.credential();
        let product = product.value();
        let currency = currency.value();
        let distribution = product
            .distributions()
            .get(distribution_index)
            .ok_or(RobotCatalogPlanError::MissingSelection)?;
        let language = product
            .languages()
            .get(language_index)
            .ok_or(RobotCatalogPlanError::MissingSelection)?;
        Ok(Self {
            product,
            currency,
            distribution,
            language,
            credential,
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
    pub const fn distribution(&self) -> &super::RobotOrderChoice {
        self.distribution
    }
    /// Returns the selected language.
    #[must_use]
    pub const fn language(&self) -> &super::RobotOrderChoice {
        self.language
    }
    /// Returns the mandatory stale-price warning.
    #[must_use]
    pub const fn price_warning(&self) -> RobotCatalogPriceWarning {
        RobotCatalogPriceWarning::RevalidateImmediatelyBeforePurchase
    }

    pub(super) const fn credential(&self) -> CredentialBinding {
        self.credential
    }
}

impl core::fmt::Debug for RobotMarketOrderPlan<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotMarketOrderPlan([redacted])")
    }
}

/// Non-executable per-server addon plan bound to one request-owned catalog.
pub struct RobotAddonOrderPlan<'catalog, 'request> {
    catalog: &'catalog RobotAddonCatalog<'request>,
    product: &'catalog RobotAddonProduct,
    currency: &'catalog RobotOrderCurrency,
    credential: CredentialBinding,
}

impl<'catalog, 'request> RobotAddonOrderPlan<'catalog, 'request> {
    /// Selects an addon from its request-bound catalog without preparing a purchase.
    pub fn new(
        catalog: &'catalog CredentialObserved<RobotAddonCatalog<'request>>,
        product_index: usize,
        currency: &'catalog CredentialObserved<RobotOrderCurrency>,
    ) -> Result<Self, RobotCatalogPlanError> {
        if !catalog.credential().matches(currency.credential()) {
            return Err(RobotCatalogPlanError::CredentialMismatch);
        }
        let credential = catalog.credential();
        let catalog = catalog.value();
        let currency = currency.value();
        let product = catalog
            .products()
            .products()
            .get(product_index)
            .ok_or(RobotCatalogPlanError::MissingSelection)?;
        Ok(Self {
            catalog,
            product,
            currency,
            credential,
        })
    }
    /// Returns the request-bound server identity.
    #[must_use]
    pub const fn server(&self) -> &RobotServerNumber {
        self.catalog.server()
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

    pub(super) const fn credential(&self) -> CredentialBinding {
        self.credential
    }
}

impl core::fmt::Debug for RobotAddonOrderPlan<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotAddonOrderPlan([redacted])")
    }
}

#[cfg(doctest)]
mod compile_fail {
    /// An addon catalog cannot be relabeled with an unrelated server identity.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::{
    ///     CredentialObserved, RobotAddonCatalog, RobotAddonOrderPlan,
    ///     RobotOrderCurrency, RobotServerNumber,
    /// };
    /// fn relabel(
    ///     catalog: &CredentialObserved<RobotAddonCatalog<'_>>,
    ///     other_server: &RobotServerNumber,
    ///     currency: &CredentialObserved<RobotOrderCurrency>,
    /// ) {
    ///     let _ = RobotAddonOrderPlan::new(other_server, catalog, 0, currency);
    /// }
    /// ```
    fn addon_catalog_server_is_not_replaceable() {}
}
