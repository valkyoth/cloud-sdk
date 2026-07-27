//! Bounded provider-neutral HTTP method tokens.

/// Maximum byte length admitted for an extension HTTP method.
pub const MAX_METHOD_BYTES: usize = 32;

/// HTTP method validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MethodError {
    /// Extension method tokens must not be empty.
    Empty,
    /// Extension method tokens exceed [`MAX_METHOD_BYTES`].
    TooLong,
    /// Extension method tokens must use uppercase canonical HTTP token bytes.
    NonCanonical,
    /// Known methods must use their dedicated [`Method`] constant.
    KnownMethod,
    /// CONNECT and TRACE are outside the SDK transport contract.
    DeniedMethod,
}

impl_static_error!(MethodError,
    Self::Empty => "HTTP extension method is empty",
    Self::TooLong => "HTTP extension method exceeds the length limit",
    Self::NonCanonical => "HTTP extension method is not a canonical uppercase token",
    Self::KnownMethod => "known HTTP method must use its dedicated constant",
    Self::DeniedMethod => "HTTP method is denied by the transport contract",
);

/// Validated HTTP method for a provider operation.
///
/// Known methods use dedicated constants. Provider extensions are admitted
/// through [`Method::extension`] and remain allocation-free.
///
/// CONNECT and TRACE are intentionally unavailable. Protocol tunnelling and
/// upgrade require a separate future transport contract.
///
/// ```compile_fail
/// use cloud_sdk::Method;
///
/// let _ = Method::Connect;
/// ```
///
/// ```compile_fail
/// use cloud_sdk::Method;
///
/// let _ = Method { token: "TRACE" };
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Method {
    token: &'static str,
}

#[allow(non_upper_case_globals)]
impl Method {
    /// GET request.
    pub const Get: Self = Self::known("GET");
    /// POST request.
    pub const Post: Self = Self::known("POST");
    /// PUT request.
    pub const Put: Self = Self::known("PUT");
    /// DELETE request.
    pub const Delete: Self = Self::known("DELETE");
    /// PATCH request.
    pub const Patch: Self = Self::known("PATCH");
    /// HEAD request.
    pub const Head: Self = Self::known("HEAD");
    /// Origin-form OPTIONS request.
    pub const Options: Self = Self::known("OPTIONS");

    /// Validates a provider extension method.
    ///
    /// The token must be a nonempty uppercase HTTP token no longer than
    /// [`MAX_METHOD_BYTES`]. Known methods are rejected so each wire method has
    /// one canonical construction path. CONNECT and TRACE are always denied.
    pub const fn extension(token: &'static str) -> Result<Self, MethodError> {
        let bytes = token.as_bytes();
        if bytes.is_empty() {
            return Err(MethodError::Empty);
        }
        if bytes.len() > MAX_METHOD_BYTES {
            return Err(MethodError::TooLong);
        }
        if token_is(bytes, b"CONNECT") || token_is(bytes, b"TRACE") {
            return Err(MethodError::DeniedMethod);
        }
        if is_known(bytes) {
            return Err(MethodError::KnownMethod);
        }

        let mut remaining = bytes;
        while let [byte, tail @ ..] = remaining {
            if !is_canonical_token_byte(*byte) {
                return Err(MethodError::NonCanonical);
            }
            remaining = tail;
        }
        Ok(Self { token })
    }

    /// Returns the canonical HTTP method token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.token
    }

    const fn known(token: &'static str) -> Self {
        Self { token }
    }
}

const fn is_known(token: &[u8]) -> bool {
    token_is(token, b"GET")
        || token_is(token, b"POST")
        || token_is(token, b"PUT")
        || token_is(token, b"DELETE")
        || token_is(token, b"PATCH")
        || token_is(token, b"HEAD")
        || token_is(token, b"OPTIONS")
}

const fn token_is(left: &[u8], right: &[u8]) -> bool {
    let mut left_remaining = left;
    let mut right_remaining = right;
    loop {
        match (left_remaining, right_remaining) {
            ([], []) => return true,
            ([left_byte, left_tail @ ..], [right_byte, right_tail @ ..]) => {
                if *left_byte != *right_byte {
                    return false;
                }
                left_remaining = left_tail;
                right_remaining = right_tail;
            }
            _ => return false,
        }
    }
}

const fn is_canonical_token_byte(byte: u8) -> bool {
    byte.is_ascii_uppercase()
        || byte.is_ascii_digit()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

#[cfg(test)]
mod tests {
    use super::{MAX_METHOD_BYTES, Method, MethodError};
    use crate::transport::{RequestTarget, RequestTargetError, TransportRequest};

    const PURGE: Method = match Method::extension("PURGE") {
        Ok(method) => method,
        Err(_) => panic!("valid extension method"),
    };

    #[test]
    fn exposes_every_admitted_known_method() {
        assert_eq!(
            [
                Method::Get.as_str(),
                Method::Post.as_str(),
                Method::Put.as_str(),
                Method::Delete.as_str(),
                Method::Patch.as_str(),
                Method::Head.as_str(),
                Method::Options.as_str(),
            ],
            ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"]
        );
    }

    #[test]
    fn admits_bounded_canonical_extension_tokens() {
        for token in [
            "PURGE",
            "PROPFIND",
            "M-SEARCH",
            "VERSION-CONTROL",
            "A!#$%&'*+-.^_`|~9",
        ] {
            assert_eq!(Method::extension(token).map(Method::as_str), Ok(token));
        }
        assert_eq!(PURGE.as_str(), "PURGE");
        assert_eq!(
            Method::extension("A2345678901234567890123456789012").map(Method::as_str),
            Ok("A2345678901234567890123456789012")
        );
    }

    #[test]
    fn rejects_empty_oversized_noncanonical_and_alias_tokens() {
        assert_eq!(Method::extension(""), Err(MethodError::Empty));
        assert_eq!(
            Method::extension("A23456789012345678901234567890123"),
            Err(MethodError::TooLong)
        );
        assert_eq!("A2345678901234567890123456789012".len(), MAX_METHOD_BYTES);

        for alias in ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"] {
            assert_eq!(Method::extension(alias), Err(MethodError::KnownMethod));
        }
        for invalid in [
            "get",
            "Get",
            "Purge",
            "M SEARCH",
            "M\tSEARCH",
            "M\r\nSEARCH",
            "M/SEARCH",
            "M:SEARCH",
            "M\\SEARCH",
            "M{SEARCH}",
            "MÜNCHEN",
        ] {
            assert_eq!(Method::extension(invalid), Err(MethodError::NonCanonical));
        }
    }

    #[test]
    fn denies_connect_trace_and_non_origin_options() {
        assert_eq!(Method::extension("CONNECT"), Err(MethodError::DeniedMethod));
        assert_eq!(Method::extension("TRACE"), Err(MethodError::DeniedMethod));
        assert_eq!(
            RequestTarget::new("*"),
            Err(RequestTargetError::Path(
                crate::transport::RequestPathError::NotOriginForm
            ))
        );

        let target = RequestTarget::new("/").unwrap_or_else(|_| unreachable!());
        let request = TransportRequest::new(Method::Options, target);
        assert_eq!(request.method(), Method::Options);
        assert_eq!(request.target().as_str(), "/");
    }
}
