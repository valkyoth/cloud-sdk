//! Read-only Robot order transaction snapshots.

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
pub use exchange::{CheckedRobotOrderTransaction, PreparedRobotOrderTransaction};
#[cfg(feature = "serde")]
pub use failure::RobotOrderTransactionFailureCode;
#[cfg(feature = "serde")]
pub use model::{
    MAX_ROBOT_ORDER_TRANSACTION_ITEMS, MAX_ROBOT_ORDER_TRANSACTION_KEYS,
    MAX_ROBOT_ORDER_TRANSACTION_RESOURCES, RobotAddonTransaction, RobotAddonTransactionList,
    RobotAddonTransactionProduct, RobotMarketTransaction, RobotMarketTransactionList,
    RobotMarketTransactionProduct, RobotOrderTransactionKey, RobotOrderTransactionResource,
    RobotOrderTransactionStatus, RobotOrderTransactionTimestamp, RobotStandardTransaction,
    RobotStandardTransactionList, RobotStandardTransactionProduct,
};

#[cfg(all(test, feature = "serde"))]
mod tests;
