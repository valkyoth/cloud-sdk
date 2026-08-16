//! Source-locked read-only Robot ordering catalogs.
//!
//! Catalog prices are observations, not durable quotes. The typed plan values
//! in this module deliberately have no transport preparation implementation.

mod prepare;
mod request;
mod transaction;
mod value;

#[cfg(feature = "serde")]
mod decode;
#[cfg(feature = "serde")]
mod exchange;
#[cfg(feature = "serde")]
mod failure;
#[cfg(feature = "serde")]
mod model;
#[cfg(feature = "serde")]
mod plan;

pub use prepare::{MAX_ROBOT_ORDER_ITEM_RESPONSE_BYTES, MAX_ROBOT_ORDER_LIST_RESPONSE_BYTES};
pub use request::{
    RobotAddonProductListRequest, RobotMarketProductGetRequest, RobotMarketProductListRequest,
    RobotOrderCurrencyRequest, RobotOrderRequestError, RobotStandardProductFilters,
    RobotStandardProductGetRequest, RobotStandardProductListRequest,
};
pub use transaction::{
    MAX_ROBOT_ORDER_TRANSACTION_ITEM_RESPONSE_BYTES,
    MAX_ROBOT_ORDER_TRANSACTION_LIST_RESPONSE_BYTES, RobotAddonTransactionGetRequest,
    RobotAddonTransactionListRequest, RobotMarketTransactionGetRequest,
    RobotMarketTransactionListRequest, RobotStandardTransactionGetRequest,
    RobotStandardTransactionListRequest,
};
pub use value::{
    MAX_ROBOT_ORDER_CHOICE_BYTES, MAX_ROBOT_ORDER_LOCATION_BYTES, MAX_ROBOT_ORDER_PRODUCT_ID_BYTES,
    MAX_ROBOT_ORDER_TRANSACTION_ID_BYTES, RobotMarketProductId, RobotOrderChoice,
    RobotOrderCurrency, RobotOrderDecimal, RobotOrderLocation, RobotOrderProductId,
    RobotOrderTransactionId, RobotOrderValueError,
};

#[cfg(feature = "serde")]
pub use decode::RobotOrderCatalogDecodeError;
#[cfg(feature = "serde")]
pub use exchange::{CheckedRobotOrderCatalog, PreparedRobotOrderCatalog, RobotAddonCatalog};
#[cfg(feature = "serde")]
pub use failure::RobotOrderFailureCode;
#[cfg(feature = "serde")]
pub use model::{
    MAX_ROBOT_ADDON_PRODUCTS, MAX_ROBOT_MARKET_PRODUCTS, MAX_ROBOT_STANDARD_PRODUCTS,
    RobotAddonProduct, RobotAddonProductList, RobotMarketProduct, RobotMarketProductList,
    RobotOrderPrice, RobotOrderPricePair, RobotOrderText, RobotOrderableAddon,
    RobotStandardProduct, RobotStandardProductList,
};
#[cfg(feature = "serde")]
pub use plan::{
    RobotAddonOrderPlan, RobotCatalogPlanError, RobotCatalogPriceWarning, RobotMarketOrderPlan,
    RobotStandardAddonSelection, RobotStandardOrderPlan,
};
#[cfg(feature = "serde")]
pub use transaction::{
    CheckedRobotOrderTransaction, MAX_ROBOT_ORDER_TRANSACTION_ITEMS,
    MAX_ROBOT_ORDER_TRANSACTION_KEYS, MAX_ROBOT_ORDER_TRANSACTION_RESOURCES,
    PreparedRobotOrderTransaction, RobotAddonTransaction, RobotAddonTransactionList,
    RobotAddonTransactionProduct, RobotMarketTransaction, RobotMarketTransactionList,
    RobotMarketTransactionProduct, RobotOrderTransactionDecodeError,
    RobotOrderTransactionFailureCode, RobotOrderTransactionKey, RobotOrderTransactionResource,
    RobotOrderTransactionStatus, RobotOrderTransactionTimestamp, RobotStandardTransaction,
    RobotStandardTransactionList, RobotStandardTransactionProduct,
};

#[cfg(all(test, feature = "serde"))]
mod tests;

#[cfg(doctest)]
mod compile_fail {
    /// Catalog plans cannot be prepared as network operations.
    ///
    /// ```compile_fail
    /// use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
    /// use cloud_sdk_hetzner::robot::RobotStandardOrderPlan;
    /// fn execute(plan: &RobotStandardOrderPlan<'_>, storage: PreparationStorage<'_>) {
    ///     let _ = plan.prepare(storage);
    /// }
    /// ```
    fn plans_are_not_executable() {}
}
