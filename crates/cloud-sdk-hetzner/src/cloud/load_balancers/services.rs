//! Load Balancer service and health-check request models.

use super::{
    HealthCheckPath, HealthCheckResponse, HealthCheckStatusCode, LoadBalancerCertificateId,
    LoadBalancerPort, LoadBalancerRequestError, StickyCookieName,
};

/// Health-check timing and retry settings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HealthCheckSettings {
    port: LoadBalancerPort,
    interval: u8,
    timeout: u8,
    retries: u8,
}

impl HealthCheckSettings {
    /// Creates settings within Hetzner's interval, timeout, and retry limits.
    pub fn new(
        port: LoadBalancerPort,
        interval: u8,
        timeout: u8,
        retries: u8,
    ) -> Result<Self, LoadBalancerRequestError> {
        if !(3..=60).contains(&interval)
            || !(1..=60).contains(&timeout)
            || !(1..=5).contains(&retries)
        {
            return Err(LoadBalancerRequestError::InvalidHealthCheck);
        }
        Ok(Self {
            port,
            interval,
            timeout,
            retries,
        })
    }

    /// Returns the health-check port.
    #[must_use]
    pub const fn port(self) -> LoadBalancerPort {
        self.port
    }

    /// Returns the interval in seconds.
    #[must_use]
    pub const fn interval(self) -> u8 {
        self.interval
    }

    /// Returns the timeout in seconds.
    #[must_use]
    pub const fn timeout(self) -> u8 {
        self.timeout
    }

    /// Returns the retry count.
    #[must_use]
    pub const fn retries(self) -> u8 {
        self.retries
    }
}

/// Optional HTTP-specific health-check settings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HttpHealthCheck<'a> {
    domain: Option<&'a str>,
    path: HealthCheckPath<'a>,
    response: Option<HealthCheckResponse<'a>>,
    status_codes: Option<&'a [HealthCheckStatusCode<'a>]>,
    tls: bool,
}

impl<'a> HttpHealthCheck<'a> {
    /// Creates an HTTP health check. `None` explicitly requests no Host header.
    pub fn try_new(
        domain: Option<&'a str>,
        path: Option<HealthCheckPath<'a>>,
    ) -> Result<Self, LoadBalancerRequestError> {
        if let Some(domain) = domain
            && (domain.len() > 128
                || domain.is_empty()
                || !domain.is_ascii()
                || domain.bytes().any(|byte| {
                    byte.is_ascii_whitespace()
                        || byte.is_ascii_control()
                        || byte == b'%'
                        || byte == b'\\'
                }))
        {
            return Err(LoadBalancerRequestError::InvalidText);
        }
        Ok(Self {
            domain,
            path: path.ok_or(LoadBalancerRequestError::MissingRequiredField)?,
            response: None,
            status_codes: None,
            tls: false,
        })
    }

    /// Sets the expected response fragment.
    #[must_use]
    pub const fn with_response(mut self, response: HealthCheckResponse<'a>) -> Self {
        self.response = Some(response);
        self
    }

    /// Sets accepted status-code patterns.
    pub fn with_status_codes(
        mut self,
        status_codes: &'a [HealthCheckStatusCode<'a>],
    ) -> Result<Self, LoadBalancerRequestError> {
        if status_codes.len() > 20 {
            return Err(LoadBalancerRequestError::TooManyItems);
        }
        self.status_codes = Some(status_codes);
        Ok(self)
    }

    /// Enables TLS for the health-check connection.
    #[must_use]
    pub const fn with_tls(mut self, tls: bool) -> Self {
        self.tls = tls;
        self
    }

    /// Returns the optional Host header, where `None` is explicit JSON null.
    #[must_use]
    pub const fn domain(self) -> Option<&'a str> {
        self.domain
    }

    /// Returns the health-check path.
    #[must_use]
    pub const fn path(self) -> HealthCheckPath<'a> {
        self.path
    }

    /// Returns the expected response fragment.
    #[must_use]
    pub const fn response(self) -> Option<HealthCheckResponse<'a>> {
        self.response
    }

    /// Returns accepted status-code patterns.
    #[must_use]
    pub const fn status_codes(self) -> Option<&'a [HealthCheckStatusCode<'a>]> {
        self.status_codes
    }

    /// Returns whether TLS is enabled.
    #[must_use]
    pub const fn tls(self) -> bool {
        self.tls
    }
}

/// Protocol-safe health-check configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadBalancerHealthCheck<'a> {
    /// TCP health check.
    Tcp(HealthCheckSettings),
    /// HTTP health check with required HTTP details.
    Http {
        /// Shared timing and retry settings.
        settings: HealthCheckSettings,
        /// HTTP-specific settings.
        http: HttpHealthCheck<'a>,
    },
}

/// Sticky-session and HTTP timeout settings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HttpServiceConfig<'a> {
    cookie_name: Option<StickyCookieName<'a>>,
    cookie_lifetime: Option<u32>,
    timeout_idle: Option<u16>,
    sticky_sessions: bool,
}

impl<'a> HttpServiceConfig<'a> {
    /// Creates default-compatible HTTP settings.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cookie_name: None,
            cookie_lifetime: None,
            timeout_idle: None,
            sticky_sessions: false,
        }
    }

    /// Sets sticky-session cookie settings.
    pub const fn with_cookie(
        mut self,
        name: StickyCookieName<'a>,
        lifetime_seconds: u32,
    ) -> Result<Self, LoadBalancerRequestError> {
        if lifetime_seconds < 30 || lifetime_seconds > 86_400 {
            return Err(LoadBalancerRequestError::InvalidServiceConfiguration);
        }
        self.cookie_name = Some(name);
        self.cookie_lifetime = Some(lifetime_seconds);
        Ok(self)
    }

    /// Sets the client/server idle timeout.
    pub const fn with_idle_timeout(
        mut self,
        seconds: u16,
    ) -> Result<Self, LoadBalancerRequestError> {
        if seconds < 30 || seconds > 300 {
            return Err(LoadBalancerRequestError::InvalidServiceConfiguration);
        }
        self.timeout_idle = Some(seconds);
        Ok(self)
    }

    /// Sets sticky-session behavior.
    #[must_use]
    pub const fn with_sticky_sessions(mut self, enabled: bool) -> Self {
        self.sticky_sessions = enabled;
        self
    }

    /// Returns the optional cookie name.
    #[must_use]
    pub const fn cookie_name(self) -> Option<StickyCookieName<'a>> {
        self.cookie_name
    }

    /// Returns the optional cookie lifetime.
    #[must_use]
    pub const fn cookie_lifetime(self) -> Option<u32> {
        self.cookie_lifetime
    }

    /// Returns the optional idle timeout.
    #[must_use]
    pub const fn timeout_idle(self) -> Option<u16> {
        self.timeout_idle
    }

    /// Returns sticky-session behavior.
    #[must_use]
    pub const fn sticky_sessions(self) -> bool {
        self.sticky_sessions
    }
}

impl Default for HttpServiceConfig<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTPS-specific settings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HttpsServiceConfig<'a> {
    http: HttpServiceConfig<'a>,
    certificates: Option<&'a [LoadBalancerCertificateId]>,
    redirect_http: bool,
}

impl<'a> HttpsServiceConfig<'a> {
    /// Creates HTTPS settings from common HTTP settings.
    #[must_use]
    pub const fn new(http: HttpServiceConfig<'a>) -> Self {
        Self {
            http,
            certificates: None,
            redirect_http: false,
        }
    }

    /// Sets certificate IDs. An empty slice explicitly requests TLS passthrough.
    #[must_use]
    pub const fn with_certificates(
        mut self,
        certificates: &'a [LoadBalancerCertificateId],
    ) -> Self {
        self.certificates = Some(certificates);
        self
    }

    /// Sets HTTP-to-HTTPS redirect behavior.
    #[must_use]
    pub const fn with_redirect_http(mut self, enabled: bool) -> Self {
        self.redirect_http = enabled;
        self
    }

    /// Returns common HTTP settings.
    #[must_use]
    pub const fn http(self) -> HttpServiceConfig<'a> {
        self.http
    }

    /// Returns certificate IDs when supplied.
    #[must_use]
    pub const fn certificates(self) -> Option<&'a [LoadBalancerCertificateId]> {
        self.certificates
    }

    /// Returns HTTP redirect behavior.
    #[must_use]
    pub const fn redirect_http(self) -> bool {
        self.redirect_http
    }
}

/// Service protocol with protocol-specific settings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadBalancerServiceProtocol<'a> {
    /// Raw TCP forwarding.
    Tcp,
    /// HTTP forwarding.
    Http(HttpServiceConfig<'a>),
    /// HTTPS termination or passthrough.
    Https(HttpsServiceConfig<'a>),
}

/// Complete service accepted by create and add-service operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerService<'a> {
    protocol: LoadBalancerServiceProtocol<'a>,
    listen_port: LoadBalancerPort,
    destination_port: LoadBalancerPort,
    proxy_protocol: bool,
    health_check: LoadBalancerHealthCheck<'a>,
}

impl<'a> LoadBalancerService<'a> {
    /// Creates a complete service.
    #[must_use]
    pub const fn new(
        protocol: LoadBalancerServiceProtocol<'a>,
        listen_port: LoadBalancerPort,
        destination_port: LoadBalancerPort,
        proxy_protocol: bool,
        health_check: LoadBalancerHealthCheck<'a>,
    ) -> Self {
        Self {
            protocol,
            listen_port,
            destination_port,
            proxy_protocol,
            health_check,
        }
    }

    /// Returns the protocol settings.
    #[must_use]
    pub const fn protocol(self) -> LoadBalancerServiceProtocol<'a> {
        self.protocol
    }
    /// Returns the listening port.
    #[must_use]
    pub const fn listen_port(self) -> LoadBalancerPort {
        self.listen_port
    }
    /// Returns the destination port.
    #[must_use]
    pub const fn destination_port(self) -> LoadBalancerPort {
        self.destination_port
    }
    /// Returns Proxy Protocol behavior.
    #[must_use]
    pub const fn proxy_protocol(self) -> bool {
        self.proxy_protocol
    }
    /// Returns health-check settings.
    #[must_use]
    pub const fn health_check(self) -> LoadBalancerHealthCheck<'a> {
        self.health_check
    }
}

/// Partial service update keyed by listening port.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerServiceUpdate<'a> {
    listen_port: LoadBalancerPort,
    protocol: Option<LoadBalancerServiceProtocol<'a>>,
    destination_port: Option<LoadBalancerPort>,
    proxy_protocol: Option<bool>,
    health_check: Option<LoadBalancerHealthCheck<'a>>,
}

impl<'a> LoadBalancerServiceUpdate<'a> {
    /// Creates an update for the required listening port.
    #[must_use]
    pub const fn new(listen_port: LoadBalancerPort) -> Self {
        Self {
            listen_port,
            protocol: None,
            destination_port: None,
            proxy_protocol: None,
            health_check: None,
        }
    }

    /// Replaces protocol-specific settings.
    #[must_use]
    pub const fn with_protocol(mut self, protocol: LoadBalancerServiceProtocol<'a>) -> Self {
        self.protocol = Some(protocol);
        self
    }
    /// Replaces the destination port.
    #[must_use]
    pub const fn with_destination_port(mut self, port: LoadBalancerPort) -> Self {
        self.destination_port = Some(port);
        self
    }
    /// Replaces Proxy Protocol behavior.
    #[must_use]
    pub const fn with_proxy_protocol(mut self, enabled: bool) -> Self {
        self.proxy_protocol = Some(enabled);
        self
    }
    /// Replaces health-check settings.
    #[must_use]
    pub const fn with_health_check(mut self, health_check: LoadBalancerHealthCheck<'a>) -> Self {
        self.health_check = Some(health_check);
        self
    }
    /// Returns the service key port.
    #[must_use]
    pub const fn listen_port(self) -> LoadBalancerPort {
        self.listen_port
    }
    /// Returns replacement protocol settings.
    #[must_use]
    pub const fn protocol(self) -> Option<LoadBalancerServiceProtocol<'a>> {
        self.protocol
    }
    /// Returns the replacement destination port.
    #[must_use]
    pub const fn destination_port(self) -> Option<LoadBalancerPort> {
        self.destination_port
    }
    /// Returns replacement Proxy Protocol behavior.
    #[must_use]
    pub const fn proxy_protocol(self) -> Option<bool> {
        self.proxy_protocol
    }
    /// Returns replacement health-check settings.
    #[must_use]
    pub const fn health_check(self) -> Option<LoadBalancerHealthCheck<'a>> {
        self.health_check
    }
}
