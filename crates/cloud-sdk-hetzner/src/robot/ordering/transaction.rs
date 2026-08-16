//! Read-only Robot order transaction snapshots.
//!
//! Transaction requests intentionally require [`PreparationStorageGuard`] and
//! do not implement the raw-storage [`PrepareOperation`] contract.
//!
//! ```compile_fail
//! use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
//! use cloud_sdk_hetzner::robot::RobotStandardTransactionListRequest;
//!
//! let request = RobotStandardTransactionListRequest::new();
//! let mut target = [0_u8; 64];
//! let mut body = [0_u8; 1];
//! let _ = request.prepare(PreparationStorage::new(&mut target, &mut body));
//! ```
//!
//! [`PreparationStorageGuard`]: cloud_sdk::operation::PreparationStorageGuard
//! [`PrepareOperation`]: cloud_sdk::operation::PrepareOperation

mod prepare;
mod request;

#[cfg(feature = "serde")]
mod decode;
#[cfg(feature = "serde")]
mod exchange;
#[cfg(feature = "serde")]
mod failure;
#[cfg(feature = "serde")]
mod model;

pub use prepare::{
    MAX_ROBOT_ORDER_TRANSACTION_ITEM_RESPONSE_BYTES,
    MAX_ROBOT_ORDER_TRANSACTION_LIST_RESPONSE_BYTES,
};
pub use request::{
    ROBOT_ORDER_TRANSACTION_QUOTA, RobotAddonTransactionGetRequest,
    RobotAddonTransactionListRequest, RobotMarketTransactionGetRequest,
    RobotMarketTransactionListRequest, RobotOrderTransactionQuota,
    RobotStandardTransactionGetRequest, RobotStandardTransactionListRequest,
};

#[cfg(feature = "serde")]
pub use decode::RobotOrderTransactionDecodeError;
#[cfg(feature = "serde")]
pub(in crate::robot::ordering) use decode::{
    decode_addon_created, decode_market_created, decode_standard,
};
#[cfg(feature = "serde")]
pub use exchange::{
    CheckedRobotOrderTransaction, CredentialCheckedRobotOrderTransaction,
    PreparedRobotOrderTransaction,
};
#[cfg(feature = "serde")]
pub use failure::RobotOrderTransactionFailureCode;
#[cfg(feature = "serde")]
pub use model::{
    MAX_ROBOT_ORDER_TRANSACTION_ITEMS, MAX_ROBOT_ORDER_TRANSACTION_KEYS,
    MAX_ROBOT_ORDER_TRANSACTION_RESOURCES, RobotAddonTransaction, RobotAddonTransactionList,
    RobotAddonTransactionProduct, RobotMarketCreatedProduct, RobotMarketCreatedTransaction,
    RobotMarketTransaction, RobotMarketTransactionList, RobotMarketTransactionProduct,
    RobotOrderTransactionKey, RobotOrderTransactionResource, RobotOrderTransactionStatus,
    RobotOrderTransactionTimestamp, RobotStandardTransaction, RobotStandardTransactionList,
    RobotStandardTransactionProduct,
};

#[cfg(all(test, feature = "serde"))]
mod tests;
