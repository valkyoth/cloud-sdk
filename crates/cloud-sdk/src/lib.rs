#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "std")]
extern crate std;

macro_rules! impl_static_error {
    ($error:ty, $($pattern:pat => $message:literal),+ $(,)?) => {
        impl core::fmt::Display for $error {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(match self {
                    $($pattern => $message,)+
                })
            }
        }

        impl core::error::Error for $error {}
    };
}

pub mod action_polling;
pub mod buffer;
pub mod operation;
pub mod pagination;
pub mod rate_limit;
pub mod transport;

/// Cloud provider namespace.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Provider {
    /// Hetzner Cloud and DNS APIs.
    Hetzner,
}

/// Provider API family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ApiFamily {
    /// Cloud infrastructure API.
    Cloud,
    /// DNS API.
    Dns,
    /// Security-related API resources.
    Security,
    /// Storage-related API resources.
    Storage,
    /// Provider-specific post-1.0 API surface.
    Extended,
}

/// HTTP method for a provider operation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Method {
    /// GET request.
    Get,
    /// POST request.
    Post,
    /// PUT request.
    Put,
    /// DELETE request.
    Delete,
}

impl Method {
    /// Returns the HTTP method token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiFamily, Method, Provider};
    use crate::action_polling::ActionPollError;
    use crate::pagination::PaginationError;
    use crate::rate_limit::RateLimitError;
    use crate::transport::{ContentTypeError, RequestTargetError};
    use core::fmt::{self, Write};

    #[test]
    fn exposes_provider_neutral_domains() {
        assert_eq!(Provider::Hetzner, Provider::Hetzner);
        assert_eq!(ApiFamily::Cloud, ApiFamily::Cloud);
        assert_eq!(Method::Get, Method::Get);
        assert_eq!(Method::Post.as_str(), "POST");
    }

    #[test]
    fn public_errors_implement_payload_free_core_error() {
        fn assert_error<E: core::error::Error>() {}

        assert_error::<PaginationError>();
        assert_error::<RateLimitError>();
        assert_error::<ContentTypeError>();
        assert_error::<RequestTargetError>();
        assert_error::<ActionPollError<&'static str>>();

        assert_display(PaginationError::PageZero, "page number must be nonzero");
        assert_display(RateLimitError::LimitZero, "rate limit must be nonzero");
        assert_display(ContentTypeError::Empty, "content type is empty");
        assert_display(RequestTargetError::Empty, "request target is empty");
        assert_display(
            ActionPollError::Policy("sentinel-secret"),
            "action poll policy failed",
        );
    }

    fn assert_display(error: impl fmt::Display, expected: &str) {
        let mut output = DisplayBuffer::new();
        assert!(write!(&mut output, "{error}").is_ok());
        assert_eq!(output.as_str(), expected);
    }

    struct DisplayBuffer {
        bytes: [u8; 128],
        len: usize,
    }

    impl DisplayBuffer {
        const fn new() -> Self {
            Self {
                bytes: [0; 128],
                len: 0,
            }
        }

        fn as_str(&self) -> &str {
            core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
        }
    }

    impl Write for DisplayBuffer {
        fn write_str(&mut self, value: &str) -> fmt::Result {
            let end = self.len.checked_add(value.len()).ok_or(fmt::Error)?;
            let target = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
            target.copy_from_slice(value.as_bytes());
            self.len = end;
            Ok(())
        }
    }
}
