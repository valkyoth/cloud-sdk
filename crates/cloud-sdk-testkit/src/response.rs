//! Deterministic response fixture builders.

use cloud_sdk::transport::StatusCode;

use crate::{ActionFixture, FixtureBody, PaginationFixture, RateLimitFixture};

/// Fixture response category.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FixtureKind {
    /// Successful response without additional metadata.
    Success,
    /// Successful paginated response.
    Pagination,
    /// Action polling response.
    Action,
    /// Rate-limit response.
    RateLimit,
    /// Client or server error response.
    Error,
}

/// Response fixture construction error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseFixtureError {
    /// Error fixtures require a `4xx` or `5xx` status.
    NonErrorStatus,
}

/// Provider-neutral response body plus optional interpreted metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResponseFixture<'a> {
    kind: FixtureKind,
    status: StatusCode,
    body: FixtureBody<'a>,
    pagination: Option<PaginationFixture>,
    action: Option<ActionFixture>,
    rate_limit: Option<RateLimitFixture>,
}

impl<'a> ResponseFixture<'a> {
    /// Creates a `200 OK` response.
    #[must_use]
    pub const fn success(body: FixtureBody<'a>) -> Self {
        Self::new(FixtureKind::Success, StatusCode::OK, body)
    }

    /// Creates a `200 OK` paginated response.
    #[must_use]
    pub const fn paginated(body: FixtureBody<'a>, pagination: PaginationFixture) -> Self {
        let mut fixture = Self::new(FixtureKind::Pagination, StatusCode::OK, body);
        fixture.pagination = Some(pagination);
        fixture
    }

    /// Creates a `200 OK` action response.
    #[must_use]
    pub const fn action(body: FixtureBody<'a>, action: ActionFixture) -> Self {
        let mut fixture = Self::new(FixtureKind::Action, StatusCode::OK, body);
        fixture.action = Some(action);
        fixture
    }

    /// Creates a `429 Too Many Requests` response.
    #[must_use]
    pub const fn rate_limited(body: FixtureBody<'a>, rate_limit: RateLimitFixture) -> Self {
        let mut fixture = Self::new(FixtureKind::RateLimit, StatusCode::TOO_MANY_REQUESTS, body);
        fixture.rate_limit = Some(rate_limit);
        fixture
    }

    /// Creates a client or server error response.
    pub const fn error(
        status: StatusCode,
        body: FixtureBody<'a>,
    ) -> Result<Self, ResponseFixtureError> {
        if !status.is_error() {
            return Err(ResponseFixtureError::NonErrorStatus);
        }
        Ok(Self::new(FixtureKind::Error, status, body))
    }

    const fn new(kind: FixtureKind, status: StatusCode, body: FixtureBody<'a>) -> Self {
        Self {
            kind,
            status,
            body,
            pagination: None,
            action: None,
            rate_limit: None,
        }
    }

    /// Returns the fixture category.
    #[must_use]
    pub const fn kind(self) -> FixtureKind {
        self.kind
    }

    /// Returns the response status.
    #[must_use]
    pub const fn status(self) -> StatusCode {
        self.status
    }

    /// Returns the response body source.
    #[must_use]
    pub const fn body(self) -> FixtureBody<'a> {
        self.body
    }

    /// Returns pagination metadata when present.
    #[must_use]
    pub const fn pagination(self) -> Option<PaginationFixture> {
        self.pagination
    }

    /// Returns action metadata when present.
    #[must_use]
    pub const fn action_metadata(self) -> Option<ActionFixture> {
        self.action
    }

    /// Returns rate-limit metadata when present.
    #[must_use]
    pub const fn rate_limit(self) -> Option<RateLimitFixture> {
        self.rate_limit
    }
}
