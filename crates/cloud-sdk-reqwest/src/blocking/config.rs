use core::fmt;
use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::HeaderValue;
use reqwest::redirect::Policy;
use reqwest::tls::Version;

use super::{BearerToken, BlockingClient, BuildError, HttpsEndpoint};

/// Maximum configured timeout accepted by the adapter.
pub const MAX_TIMEOUT_SECONDS: u64 = 300;

/// Timeout policy validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeoutError {
    /// Every timeout must be nonzero.
    Zero,
    /// Every timeout is capped at [`MAX_TIMEOUT_SECONDS`].
    TooLong,
    /// The connect timeout must not exceed the total timeout.
    ExceedsTotal,
}

/// Explicit total-request and connection timeout policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestTimeouts {
    total: Duration,
    connect: Duration,
}

impl RequestTimeouts {
    /// Validates all timeout dimensions.
    pub fn new(total: Duration, connect: Duration) -> Result<Self, TimeoutError> {
        if total.is_zero() || connect.is_zero() {
            return Err(TimeoutError::Zero);
        }
        let maximum = Duration::from_secs(MAX_TIMEOUT_SECONDS);
        if total > maximum || connect > maximum {
            return Err(TimeoutError::TooLong);
        }
        if connect > total {
            return Err(TimeoutError::ExceedsTotal);
        }
        Ok(Self { total, connect })
    }

    pub(super) const fn total(self) -> Duration {
        self.total
    }

    pub(super) const fn connect(self) -> Duration {
        self.connect
    }
}

/// User-agent validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserAgentError {
    /// User agents must not be empty.
    Empty,
    /// User agents are capped at 256 bytes.
    TooLong,
    /// The value is not a valid HTTP header value.
    Invalid,
}

/// Validated, non-secret user-agent header value.
#[derive(Clone)]
pub struct UserAgent {
    value: HeaderValue,
}

impl UserAgent {
    /// Validates a user-agent value.
    pub fn new(value: &str) -> Result<Self, UserAgentError> {
        if value.is_empty() {
            return Err(UserAgentError::Empty);
        }
        if value.len() > 256 {
            return Err(UserAgentError::TooLong);
        }
        let value = HeaderValue::from_str(value).map_err(|_| UserAgentError::Invalid)?;
        Ok(Self { value })
    }
}

impl fmt::Debug for UserAgent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("UserAgent([validated])")
    }
}

/// Builder requiring endpoint, bearer token, user agent, and all timeout
/// dimensions before a client can be constructed.
pub struct BlockingClientBuilder {
    endpoint: HttpsEndpoint,
    token: BearerToken,
    user_agent: UserAgent,
    timeouts: RequestTimeouts,
}

impl BlockingClientBuilder {
    /// Creates a complete blocking-client configuration.
    #[must_use]
    pub const fn new(
        endpoint: HttpsEndpoint,
        token: BearerToken,
        user_agent: UserAgent,
        timeouts: RequestTimeouts,
    ) -> Self {
        Self {
            endpoint,
            token,
            user_agent,
            timeouts,
        }
    }

    /// Builds a hardened HTTPS-only client.
    pub fn build(self) -> Result<BlockingClient, BuildError> {
        self.build_inner(true)
    }

    fn build_inner(self, https_only: bool) -> Result<BlockingClient, BuildError> {
        let client = configured_client(&self.user_agent, self.timeouts, https_only)?;
        Ok(BlockingClient::new(client, self.endpoint, self.token))
    }

    #[cfg(test)]
    pub(super) fn build_for_loopback(self) -> Result<BlockingClient, BuildError> {
        self.build_inner(false)
    }
}

impl fmt::Debug for BlockingClientBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BlockingClientBuilder")
            .field("endpoint", &"[redacted]")
            .field("token", &"[redacted]")
            .field("user_agent", &self.user_agent)
            .field("timeouts", &self.timeouts)
            .finish()
    }
}

fn configured_client(
    user_agent: &UserAgent,
    timeouts: RequestTimeouts,
    https_only: bool,
) -> Result<Client, BuildError> {
    Client::builder()
        .use_rustls_tls()
        .https_only(https_only)
        .min_tls_version(Version::TLS_1_2)
        .redirect(Policy::none())
        .retry(reqwest::retry::never())
        .referer(false)
        .no_proxy()
        .no_gzip()
        .no_brotli()
        .no_zstd()
        .no_deflate()
        .timeout(timeouts.total())
        .connect_timeout(timeouts.connect())
        .connection_verbose(false)
        .user_agent(user_agent.value.clone())
        .build()
        .map_err(|_| BuildError::ClientBuildFailed)
}
