use core::fmt;
use reqwest::Url;
use std::string::String;
#[cfg(test)]
use std::string::ToString;

use cloud_sdk::transport::RequestTarget;

/// HTTPS endpoint validation or target-composition error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointError {
    /// The endpoint is not a valid absolute URL.
    InvalidUrl,
    /// Public endpoints must use HTTPS.
    HttpsRequired,
    /// A host is required.
    MissingHost,
    /// URL user information is forbidden.
    CredentialsForbidden,
    /// Base endpoint queries are forbidden.
    QueryForbidden,
    /// Base endpoint fragments are forbidden.
    FragmentForbidden,
    /// Non-root endpoint paths must not end in `/`.
    TrailingSlash,
    /// Target percent encoding is malformed or encodes structural path bytes.
    InvalidTargetEncoding,
    /// URL parsing changed the exact configured base plus target bytes.
    TargetNormalized,
    /// Allocation failed during target composition.
    AllocationFailed,
}

impl_static_error!(EndpointError,
    Self::InvalidUrl => "endpoint URL is invalid",
    Self::HttpsRequired => "endpoint must use HTTPS",
    Self::MissingHost => "endpoint host is missing",
    Self::CredentialsForbidden => "endpoint credentials are forbidden",
    Self::QueryForbidden => "endpoint query is forbidden",
    Self::FragmentForbidden => "endpoint fragment is forbidden",
    Self::TrailingSlash => "endpoint path has a forbidden trailing slash",
    Self::InvalidTargetEncoding => "request target encoding is invalid",
    Self::TargetNormalized => "request target was normalized or changed origin",
    Self::AllocationFailed => "request-target allocation failed",
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
    /// Explicitly trusts and validates a custom HTTPS credential destination.
    ///
    /// The transport sends its bearer token to this origin. `value` must come
    /// from trusted operator configuration and must never be controlled by a
    /// tenant, request payload, or other untrusted input.
    pub fn new_custom(value: &str) -> Result<Self, EndpointError> {
        Self::new_inner(value, true)
    }

    fn new_inner(value: &str, require_https: bool) -> Result<Self, EndpointError> {
        let base = Url::parse(value).map_err(|_| EndpointError::InvalidUrl)?;
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
        Ok(Self { base, prefix })
    }

    pub(crate) fn compose(&self, target: RequestTarget<'_>) -> Result<Url, EndpointError> {
        validate_target_encoding(target.as_str())?;
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

fn validate_target_encoding(target: &str) -> Result<(), EndpointError> {
    let path_end = target.find('?').unwrap_or(target.len());
    let bytes = target.as_bytes();
    let mut index = 0_usize;
    while index < bytes.len() {
        if bytes.get(index) != Some(&b'%') {
            index = index
                .checked_add(1)
                .ok_or(EndpointError::InvalidTargetEncoding)?;
            continue;
        }
        let high = bytes
            .get(
                index
                    .checked_add(1)
                    .ok_or(EndpointError::InvalidTargetEncoding)?,
            )
            .and_then(|byte| decode_hex(*byte));
        let low = bytes
            .get(
                index
                    .checked_add(2)
                    .ok_or(EndpointError::InvalidTargetEncoding)?,
            )
            .and_then(|byte| decode_hex(*byte));
        let (Some(high), Some(low)) = (high, low) else {
            return Err(EndpointError::InvalidTargetEncoding);
        };
        let decoded = high
            .checked_mul(16)
            .and_then(|value| value.checked_add(low))
            .ok_or(EndpointError::InvalidTargetEncoding)?;
        if index < path_end
            && (decoded <= b' '
                || decoded >= 0x7f
                || matches!(decoded, b'.' | b'/' | b'\\' | b'?' | b'#' | b'%'))
        {
            return Err(EndpointError::InvalidTargetEncoding);
        }
        index = index
            .checked_add(3)
            .ok_or(EndpointError::InvalidTargetEncoding)?;
    }
    Ok(())
}

const fn decode_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0'),
        b'a'..=b'f' => match byte.checked_sub(b'a') {
            Some(value) => value.checked_add(10),
            None => None,
        },
        b'A'..=b'F' => match byte.checked_sub(b'A') {
            Some(value) => value.checked_add(10),
            None => None,
        },
        _ => None,
    }
}
