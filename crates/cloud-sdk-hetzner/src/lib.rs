#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

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

pub mod actions;
pub mod cloud;
pub mod dns;
pub mod endpoint;
pub mod labels;
pub mod pagination;
pub mod query;
pub mod rate_limit;
pub mod request;
pub mod response;
pub mod security;
#[cfg(feature = "serde")]
pub mod serde;
pub mod storage;

pub use endpoint::{ApiSurface, EndpointGroup, OfficialEndpointError, verify_official_endpoint};
pub use request::{CLOUD_API_BASE_URL, CLOUD_API_VERSION};

#[cfg(test)]
mod tests {
    use super::{ApiSurface, EndpointGroup, OfficialEndpointError};
    use crate::actions::ActionRequestError;
    use crate::cloud::catalog::CatalogRequestError;
    use crate::cloud::firewalls::rules::FirewallRuleError;
    use crate::cloud::ip::IpValidationError;
    use crate::cloud::load_balancers::LoadBalancerRequestError;
    use crate::cloud::servers::ServerRequestError;
    use crate::cloud::shared::CloudRequestError;
    use crate::dns::rrsets::RrsetRequestError;
    use crate::dns::zones::ZoneRequestError;
    use crate::labels::LabelError;
    use crate::pagination::{PaginationError, SortError};
    use crate::query::QueryError;
    use crate::request::EndpointPathError;
    use crate::response::ApiError;
    use crate::security::ssh_keys::SecurityRequestError;
    #[cfg(feature = "serde")]
    use crate::serde::{ApiErrorResponse, ResponseSizeError, RrsetBodyError};
    use core::fmt::{self, Write};

    #[test]
    fn sdk_exposes_endpoint_domains() {
        assert_eq!(EndpointGroup::Firewalls.surface(), ApiSurface::Cloud);
        assert_eq!(EndpointGroup::ZoneActions.surface(), ApiSurface::Dns);
        assert_eq!(EndpointGroup::StorageBoxes.surface(), ApiSurface::Storage);
    }

    #[test]
    fn public_errors_implement_payload_free_core_error() {
        fn assert_error<E: core::error::Error>() {}

        assert_error::<ActionRequestError>();
        assert_error::<CatalogRequestError>();
        assert_error::<CloudRequestError>();
        assert_error::<EndpointPathError>();
        assert_error::<FirewallRuleError>();
        assert_error::<IpValidationError>();
        assert_error::<LabelError>();
        assert_error::<LoadBalancerRequestError>();
        assert_error::<OfficialEndpointError>();
        assert_error::<PaginationError>();
        assert_error::<QueryError>();
        assert_error::<RrsetRequestError>();
        assert_error::<SecurityRequestError>();
        assert_error::<ServerRequestError>();
        assert_error::<SortError>();
        assert_error::<ZoneRequestError>();
        assert_error::<ApiError<'static>>();

        assert_display(
            ActionRequestError::InvalidPath(EndpointPathError::Empty),
            "action endpoint path is invalid",
        );
        assert_display(
            CatalogRequestError::UnsupportedSorting,
            "catalog endpoint does not support sorting",
        );
        assert_display(
            CloudRequestError::InvalidName,
            "cloud resource name is invalid",
        );
        assert_display(EndpointPathError::Empty, "endpoint path is empty");
        assert_display(
            FirewallRuleError::InvalidPort,
            "firewall rule port is invalid",
        );
        assert_display(IpValidationError::HostBitsSet, "CIDR has host bits set");
        assert_display(LabelError::EmptyKey, "label key is empty");
        assert_display(
            LoadBalancerRequestError::InvalidPort,
            "load-balancer port is invalid",
        );
        assert_display(
            OfficialEndpointError::DestinationMismatch,
            "transport destination is not an official Hetzner endpoint",
        );
        assert_display(PaginationError::PageZero, "page number must be nonzero");
        assert_display(QueryError::EmptyKey, "query key is empty");
        assert_display(RrsetRequestError::InvalidName, "RRSet name is invalid");
        assert_display(
            SecurityRequestError::EmptyName,
            "security resource name is empty",
        );
        assert_display(
            ServerRequestError::InvalidUserData,
            "server user data is invalid",
        );
        assert_display(SortError::Empty, "sort key is empty");
        assert_display(
            ZoneRequestError::InvalidZoneName,
            "DNS zone name is invalid",
        );
        assert_display(
            ApiError::new(
                crate::response::ApiErrorCode::InvalidInput,
                "sentinel-secret",
            ),
            "Hetzner API returned an error",
        );
        #[cfg(feature = "serde")]
        {
            assert_error::<ApiErrorResponse<'static>>();
            assert_error::<ResponseSizeError>();
            assert_error::<RrsetBodyError>();
            assert_display(
                ResponseSizeError::TooLarge,
                "provider response exceeds the parser size limit",
            );
            assert_display(
                RrsetBodyError::SizeOverflow,
                "RRSet JSON size calculation overflowed",
            );
        }
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
