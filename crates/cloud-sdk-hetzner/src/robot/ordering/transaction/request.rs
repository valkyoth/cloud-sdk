use super::super::RobotOrderTransactionId;

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
