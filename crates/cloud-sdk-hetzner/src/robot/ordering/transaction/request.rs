use cloud_sdk::rate_limit::DelaySeconds;

use super::super::RobotOrderTransactionId;

/// Source-locked allowance shared by all Robot transaction operations.
///
/// This is one account-level budget. It is not a separate allowance for each
/// request type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RobotOrderTransactionQuota {
    max_requests: u16,
    interval: DelaySeconds,
}

impl RobotOrderTransactionQuota {
    /// Returns the documented request allowance.
    #[must_use]
    pub const fn max_requests(self) -> u16 {
        self.max_requests
    }

    /// Returns the documented quota interval.
    #[must_use]
    pub const fn interval(self) -> DelaySeconds {
        self.interval
    }
}

/// Five hundred transaction requests per shared one-hour account window.
pub const ROBOT_ORDER_TRANSACTION_QUOTA: RobotOrderTransactionQuota = RobotOrderTransactionQuota {
    max_requests: 500,
    interval: DelaySeconds::new(3_600),
};

macro_rules! expose_quota {
    ($($request:ty),+ $(,)?) => {
        $(
            impl $request {
                /// Returns the shared source-locked transaction allowance.
                #[must_use]
                pub const fn quota(&self) -> RobotOrderTransactionQuota {
                    ROBOT_ORDER_TRANSACTION_QUOTA
                }
            }
        )+
    };
}

/// Lists standard-server order transactions from Robot's fixed 30-day window.
#[derive(Clone, Copy, Debug, Default)]
pub struct RobotStandardTransactionListRequest;

impl RobotStandardTransactionListRequest {
    /// Creates a standard-server transaction snapshot request.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// Gets one standard-server order transaction.
#[derive(Debug)]
pub struct RobotStandardTransactionGetRequest {
    pub(super) id: RobotOrderTransactionId,
}

impl RobotStandardTransactionGetRequest {
    /// Creates a request for one exact transaction identifier.
    #[must_use]
    pub const fn new(id: RobotOrderTransactionId) -> Self {
        Self { id }
    }
}

/// Lists Server Auction order transactions from Robot's fixed 30-day window.
#[derive(Clone, Copy, Debug, Default)]
pub struct RobotMarketTransactionListRequest;

impl RobotMarketTransactionListRequest {
    /// Creates a Server Auction transaction snapshot request.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// Gets one Server Auction order transaction.
#[derive(Debug)]
pub struct RobotMarketTransactionGetRequest {
    pub(super) id: RobotOrderTransactionId,
}

impl RobotMarketTransactionGetRequest {
    /// Creates a request for one exact transaction identifier.
    #[must_use]
    pub const fn new(id: RobotOrderTransactionId) -> Self {
        Self { id }
    }
}

/// Lists per-server addon transactions from Robot's fixed 30-day window.
#[derive(Clone, Copy, Debug, Default)]
pub struct RobotAddonTransactionListRequest;

impl RobotAddonTransactionListRequest {
    /// Creates an addon transaction snapshot request.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// Gets one per-server addon transaction.
#[derive(Debug)]
pub struct RobotAddonTransactionGetRequest {
    pub(super) id: RobotOrderTransactionId,
}

impl RobotAddonTransactionGetRequest {
    /// Creates a request for one exact transaction identifier.
    #[must_use]
    pub const fn new(id: RobotOrderTransactionId) -> Self {
        Self { id }
    }
}

expose_quota!(
    RobotStandardTransactionListRequest,
    RobotStandardTransactionGetRequest,
    RobotMarketTransactionListRequest,
    RobotMarketTransactionGetRequest,
    RobotAddonTransactionListRequest,
    RobotAddonTransactionGetRequest,
);
