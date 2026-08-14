use crate::robot::RobotServerNumber;

use super::{
    RobotMarketProductId, RobotOrderDecimal, RobotOrderLocation, RobotOrderProductId,
    RobotOrderValueError,
};

/// Failure while validating or preparing a Robot ordering-catalog request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotOrderRequestError {
    /// A typed request value was invalid.
    Value(RobotOrderValueError),
    /// A minimum filter exceeded its corresponding maximum.
    InvalidPriceRange,
    /// Caller-owned target storage was too small or encoding failed.
    Target,
    /// The constructed request target was rejected.
    InvalidTarget(cloud_sdk::transport::RequestTargetError),
    /// Source-locked request headers were rejected.
    InvalidHeaders(cloud_sdk::transport::HeaderError),
    /// The official Robot endpoint policy was invalid.
    InvalidEndpoint(crate::endpoint::OfficialEndpointError),
    /// A source-locked operation identifier was invalid.
    InvalidOperationId(cloud_sdk::operation::OperationIdError),
    /// Operation safety metadata was internally inconsistent.
    InvalidMetadata(cloud_sdk::operation::OperationMetadataError),
    /// The success-response policy was internally inconsistent.
    InvalidResponsePolicy(cloud_sdk::operation::ResponsePolicyValidationError),
    /// The raw response-wire policy was internally inconsistent.
    InvalidRawPolicy(cloud_sdk::transport::RawResponsePolicyError),
    /// Cross-policy prepared-request validation failed.
    InvalidPreparedPolicy(cloud_sdk::operation::PreparedRequestPolicyError),
}

impl_static_error!(RobotOrderRequestError,
    Self::Value(_) => "Robot order catalog value is invalid",
    Self::InvalidPriceRange => "Robot order catalog price range is invalid",
    Self::Target => "Robot order catalog target preparation failed",
    Self::InvalidTarget(_) => "Robot order catalog target is invalid",
    Self::InvalidHeaders(_) => "Robot order catalog headers are invalid",
    Self::InvalidEndpoint(_) => "official Robot endpoint is invalid",
    Self::InvalidOperationId(_) => "Robot order catalog operation identifier is invalid",
    Self::InvalidMetadata(_) => "Robot order catalog metadata is invalid",
    Self::InvalidResponsePolicy(_) => "Robot order catalog response policy is invalid",
    Self::InvalidRawPolicy(_) => "Robot order catalog raw response policy is invalid",
    Self::InvalidPreparedPolicy(_) => "Robot order catalog prepared policy is invalid",
);

/// Optional source-locked filters for standard-server products.
#[derive(Default)]
pub struct RobotStandardProductFilters {
    pub(super) min_price: Option<RobotOrderDecimal>,
    pub(super) max_price: Option<RobotOrderDecimal>,
    pub(super) min_setup: Option<RobotOrderDecimal>,
    pub(super) max_setup: Option<RobotOrderDecimal>,
    pub(super) location: Option<RobotOrderLocation>,
}

impl RobotStandardProductFilters {
    /// Validates monthly and setup price intervals.
    pub fn new(
        min_price: Option<RobotOrderDecimal>,
        max_price: Option<RobotOrderDecimal>,
        min_setup: Option<RobotOrderDecimal>,
        max_setup: Option<RobotOrderDecimal>,
        location: Option<RobotOrderLocation>,
    ) -> Result<Self, RobotOrderRequestError> {
        if exceeds(&min_price, &max_price) || exceeds(&min_setup, &max_setup) {
            return Err(RobotOrderRequestError::InvalidPriceRange);
        }
        Ok(Self {
            min_price,
            max_price,
            min_setup,
            max_setup,
            location,
        })
    }

    /// Reports whether no server-product filter is present.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.min_price.is_none()
            && self.max_price.is_none()
            && self.min_setup.is_none()
            && self.max_setup.is_none()
            && self.location.is_none()
    }
}

impl core::fmt::Debug for RobotStandardProductFilters {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotStandardProductFilters")
            .field("min_price", &self.min_price.is_some())
            .field("max_price", &self.max_price.is_some())
            .field("min_setup", &self.min_setup.is_some())
            .field("max_setup", &self.max_setup.is_some())
            .field("location", &self.location.is_some())
            .finish()
    }
}

fn exceeds(minimum: &Option<RobotOrderDecimal>, maximum: &Option<RobotOrderDecimal>) -> bool {
    matches!((minimum, maximum), (Some(minimum), Some(maximum)) if minimum > maximum)
}

/// Lists currently offered standard-server products.
#[derive(Debug, Default)]
pub struct RobotStandardProductListRequest {
    pub(super) filters: RobotStandardProductFilters,
}

impl RobotStandardProductListRequest {
    /// Creates a request with explicit optional filters.
    #[must_use]
    pub const fn new(filters: RobotStandardProductFilters) -> Self {
        Self { filters }
    }
}

/// Gets one standard-server product.
#[derive(Debug)]
pub struct RobotStandardProductGetRequest {
    pub(super) id: RobotOrderProductId,
}

impl RobotStandardProductGetRequest {
    /// Creates a standard-product detail request.
    #[must_use]
    pub const fn new(id: RobotOrderProductId) -> Self {
        Self { id }
    }
}

/// Lists currently offered Server Auction products.
#[derive(Clone, Copy, Debug, Default)]
pub struct RobotMarketProductListRequest;

impl RobotMarketProductListRequest {
    /// Creates a Server Auction inventory request.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// Gets one Server Auction product.
#[derive(Clone, Copy, Debug)]
pub struct RobotMarketProductGetRequest {
    pub(super) id: RobotMarketProductId,
}

impl RobotMarketProductGetRequest {
    /// Creates a Server Auction product detail request.
    #[must_use]
    pub const fn new(id: RobotMarketProductId) -> Self {
        Self { id }
    }
}

/// Lists addons currently available for one server.
#[derive(Debug)]
pub struct RobotAddonProductListRequest {
    pub(super) server: RobotServerNumber,
}

impl RobotAddonProductListRequest {
    /// Creates a server-addon catalog request.
    #[must_use]
    pub const fn new(server: RobotServerNumber) -> Self {
        Self { server }
    }
}

/// Reads the authenticated Robot account currency.
#[derive(Clone, Copy, Debug, Default)]
pub struct RobotOrderCurrencyRequest;

impl RobotOrderCurrencyRequest {
    /// Creates an account-currency request.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
