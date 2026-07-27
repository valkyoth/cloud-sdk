use core::fmt;
use reqwest::Url;
use std::string::String;
#[cfg(test)]
use std::string::ToString;

use cloud_sdk::transport::{
    AcknowledgedCustomEndpoint, CustomEndpointAcknowledgement, EndpointIdentity,
    EndpointIdentityError, EndpointPolicy, EndpointScheme, MAX_ENDPOINT_BASE_PATH_BYTES,
    MAX_ENDPOINT_HOST_BYTES, RequestTarget,
};

/// Maximum endpoint input accepted before URL parsing or allocation.
///
/// This covers `https://`, a maximum-length host, `:65535`, and a
/// maximum-length base path.
pub const MAX_CONFIGURED_ENDPOINT_BYTES: usize =
    "https://".len() + MAX_ENDPOINT_HOST_BYTES + 1 + 5 + MAX_ENDPOINT_BASE_PATH_BYTES;

/// HTTPS endpoint validation or target-composition error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointError {
    /// The raw endpoint exceeds [`MAX_CONFIGURED_ENDPOINT_BYTES`].
    InputTooLong,
    /// The endpoint is not a valid absolute URL.
    InvalidUrl,
    /// Public endpoints must use HTTPS.
    HttpsRequired,
    /// A host is required.
    MissingHost,
    /// URL user information is forbidden.
    CredentialsForbidden,
    /// The configured authority uses a non-canonical or ambiguous host form.
    AmbiguousAuthority,
    /// Base endpoint queries are forbidden.
    QueryForbidden,
    /// Base endpoint fragments are forbidden.
    FragmentForbidden,
    /// Non-root endpoint paths must not end in `/`.
    TrailingSlash,
    /// URL parsing changed the exact configured base plus target bytes.
    TargetNormalized,
    /// Allocation failed during target composition.
    AllocationFailed,
    /// The normalized endpoint could not be represented by the shared identity contract.
    IdentityRejected,
    /// The endpoint is not admitted by the supplied provider policy.
    PolicyRejected,
}

impl_static_error!(EndpointError,
    Self::InputTooLong => "endpoint input exceeds the length limit",
    Self::InvalidUrl => "endpoint URL is invalid",
    Self::HttpsRequired => "endpoint must use HTTPS",
    Self::MissingHost => "endpoint host is missing",
    Self::CredentialsForbidden => "endpoint credentials are forbidden",
    Self::AmbiguousAuthority => "endpoint authority is ambiguous or non-canonical",
    Self::QueryForbidden => "endpoint query is forbidden",
    Self::FragmentForbidden => "endpoint fragment is forbidden",
    Self::TrailingSlash => "endpoint path has a forbidden trailing slash",
    Self::TargetNormalized => "request target was normalized or changed origin",
    Self::AllocationFailed => "request-target allocation failed",
    Self::IdentityRejected => "endpoint identity is invalid",
    Self::PolicyRejected => "endpoint is not admitted by provider policy",
);

/// Validated HTTPS API endpoint, optionally including a fixed base path.
#[derive(Clone)]
pub struct HttpsEndpoint {
    base: Url,
    prefix: String,
}

impl fmt::Debug for HttpsEndpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("HttpsEndpoint([redacted])")
    }
}

impl HttpsEndpoint {
    /// Validates an endpoint and requires admission by a provider-owned policy.
    ///
    /// This is the preferred constructor for official and region-derived
    /// provider destinations.
    pub fn new_with_policy(value: &str, policy: EndpointPolicy<'_>) -> Result<Self, EndpointError> {
        let endpoint = Self::new_inner(value, true)?;
        policy
            .verify(
                endpoint
                    .identity()
                    .map_err(|_| EndpointError::IdentityRejected)?,
            )
            .map_err(|_| EndpointError::PolicyRejected)?;
        Ok(endpoint)
    }

    /// Explicitly trusts and validates a custom HTTPS credential destination.
    ///
    /// The transport sends its bearer token to this origin. `value` must come
    /// from trusted operator configuration and must never be controlled by a
    /// tenant, request payload, or other untrusted input.
    ///
    /// ```compile_fail
    /// use cloud_sdk_reqwest::blocking::HttpsEndpoint;
    ///
    /// let endpoint = HttpsEndpoint::new_custom("https://api.example.test/v1");
    /// ```
    pub fn new_custom(
        value: &str,
        acknowledgement: CustomEndpointAcknowledgement,
    ) -> Result<Self, EndpointError> {
        let endpoint = Self::new_inner(value, true)?;
        let identity = endpoint
            .identity()
            .map_err(|_| EndpointError::IdentityRejected)?;
        EndpointPolicy::acknowledged_custom(AcknowledgedCustomEndpoint::new(
            identity,
            acknowledgement,
        ))
        .verify(identity)
        .map_err(|_| EndpointError::PolicyRejected)?;
        Ok(endpoint)
    }

    fn new_inner(value: &str, require_https: bool) -> Result<Self, EndpointError> {
        if value.len() > MAX_CONFIGURED_ENDPOINT_BYTES {
            return Err(EndpointError::InputTooLong);
        }
        validate_raw_authority(value)?;
        let configured_path = validate_configured_base_path(value)?;
        let base = Url::parse(value).map_err(|_| EndpointError::InvalidUrl)?;
        if base.path() != configured_path {
            return Err(EndpointError::TargetNormalized);
        }
        if require_https && base.scheme() != "https" {
            return Err(EndpointError::HttpsRequired);
        }
        if base.host_str().is_none() {
            return Err(EndpointError::MissingHost);
        }
        if !base.username().is_empty() || base.password().is_some() {
            return Err(EndpointError::CredentialsForbidden);
        }
        if base.query().is_some() {
            return Err(EndpointError::QueryForbidden);
        }
        if base.fragment().is_some() {
            return Err(EndpointError::FragmentForbidden);
        }
        if base.path() != "/" && base.path().ends_with('/') {
            return Err(EndpointError::TrailingSlash);
        }

        let mut prefix = String::from(base.as_str());
        if base.path() == "/" {
            prefix.pop();
        }
        let endpoint = Self { base, prefix };
        endpoint
            .identity()
            .map_err(|_| EndpointError::IdentityRejected)?;
        Ok(endpoint)
    }

    /// Returns the normalized scheme, host, effective port, and base path.
    pub fn identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        let scheme = match self.base.scheme() {
            "https" => EndpointScheme::Https,
            "http" => EndpointScheme::Http,
            _ => return Err(EndpointIdentityError::InvalidBasePath),
        };
        let host = self
            .base
            .host_str()
            .ok_or(EndpointIdentityError::EmptyHost)?;
        let port = self
            .base
            .port_or_known_default()
            .ok_or(EndpointIdentityError::InvalidPort)?;
        EndpointIdentity::new(scheme, host, port, self.base.path())
    }

    pub(crate) fn compose(&self, target: RequestTarget<'_>) -> Result<Url, EndpointError> {
        let mut absolute = self.prefix.clone();
        absolute
            .try_reserve_exact(target.as_str().len())
            .map_err(|_| EndpointError::AllocationFailed)?;
        absolute.push_str(target.as_str());
        let url = Url::parse(&absolute).map_err(|_| EndpointError::InvalidUrl)?;
        if url.as_str() != absolute {
            return Err(EndpointError::TargetNormalized);
        }
        self.verify_origin(&url)?;
        Ok(url)
    }

    pub(crate) fn verify_origin(&self, url: &Url) -> Result<(), EndpointError> {
        if url.scheme() != self.base.scheme()
            || url.host_str() != self.base.host_str()
            || url.port_or_known_default() != self.base.port_or_known_default()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(EndpointError::TargetNormalized);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn local_http(value: &str) -> Result<Self, EndpointError> {
        let endpoint = Self::new_inner(value, false)?;
        if endpoint.base.scheme() != "http"
            || !endpoint.base.host().is_some_and(|host| {
                host.to_string()
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
            })
        {
            return Err(EndpointError::HttpsRequired);
        }
        Ok(endpoint)
    }
}

fn validate_raw_authority(value: &str) -> Result<(), EndpointError> {
    let (scheme, remainder) = value.split_once("://").ok_or(EndpointError::InvalidUrl)?;
    if scheme.is_empty()
        || !scheme.is_ascii()
        || scheme.bytes().any(|byte| byte.is_ascii_uppercase())
    {
        return Err(EndpointError::AmbiguousAuthority);
    }
    let authority_end = remainder.find(['/', '?', '#']).unwrap_or(remainder.len());
    let authority = remainder
        .get(..authority_end)
        .ok_or(EndpointError::AmbiguousAuthority)?;
    if authority.is_empty() {
        return Err(EndpointError::MissingHost);
    }
    if authority.contains('@') {
        return Err(EndpointError::CredentialsForbidden);
    }
    if !authority.is_ascii() || authority.contains(['%', '\\']) {
        return Err(EndpointError::AmbiguousAuthority);
    }

    let (host, port) = split_host_port(authority)?;
    if host.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return Err(EndpointError::AmbiguousAuthority);
    }
    if let Some(port) = port
        && (port.is_empty()
            || !port.bytes().all(|byte| byte.is_ascii_digit())
            || port.len() > 1 && port.starts_with('0')
            || port
                .parse::<u16>()
                .ok()
                .filter(|value| *value != 0)
                .is_none())
    {
        return Err(EndpointError::AmbiguousAuthority);
    }
    EndpointIdentity::new(EndpointScheme::Https, host, 1, "/")
        .map(|_| ())
        .map_err(|_| EndpointError::AmbiguousAuthority)
}

fn split_host_port(authority: &str) -> Result<(&str, Option<&str>), EndpointError> {
    if authority.starts_with('[') {
        let close = authority
            .find(']')
            .ok_or(EndpointError::AmbiguousAuthority)?;
        let host_end = close
            .checked_add(1)
            .ok_or(EndpointError::AmbiguousAuthority)?;
        let host = authority
            .get(..host_end)
            .ok_or(EndpointError::AmbiguousAuthority)?;
        let suffix = authority
            .get(host_end..)
            .ok_or(EndpointError::AmbiguousAuthority)?;
        if suffix.is_empty() {
            return Ok((host, None));
        }
        let port = suffix
            .strip_prefix(':')
            .ok_or(EndpointError::AmbiguousAuthority)?;
        return Ok((host, Some(port)));
    }
    if authority.matches(':').count() > 1 {
        return Err(EndpointError::AmbiguousAuthority);
    }
    Ok(match authority.rsplit_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (authority, None),
    })
}

fn validate_configured_base_path(value: &str) -> Result<&str, EndpointError> {
    let (_, authority_and_path) = value.split_once("://").ok_or(EndpointError::InvalidUrl)?;
    let suffix_start = authority_and_path
        .find(['/', '?', '#'])
        .unwrap_or(authority_and_path.len());
    let suffix = authority_and_path
        .get(suffix_start..)
        .ok_or(EndpointError::IdentityRejected)?;
    let path = if suffix.starts_with('/') {
        let path_end = suffix.find(['?', '#']).unwrap_or(suffix.len());
        suffix
            .get(..path_end)
            .ok_or(EndpointError::IdentityRejected)?
    } else {
        "/"
    };
    if path.len() > MAX_ENDPOINT_BASE_PATH_BYTES
        || path.contains("//")
        || path.split('/').any(|part| matches!(part, "." | ".."))
        || !path
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && !matches!(byte, b'\\' | b'%' | b'?' | b'#'))
    {
        return Err(EndpointError::IdentityRejected);
    }
    Ok(path)
}
