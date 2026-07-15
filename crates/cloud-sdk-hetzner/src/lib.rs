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

pub use endpoint::{ApiSurface, EndpointGroup};
pub use request::{CLOUD_API_BASE_URL, CLOUD_API_VERSION};

#[cfg(test)]
mod tests {
    use super::{ApiSurface, EndpointGroup};
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
        assert_error::<PaginationError>();
        assert_error::<QueryError>();
        assert_error::<RrsetRequestError>();
        assert_error::<SecurityRequestError>();
        assert_error::<ServerRequestError>();
        assert_error::<SortError>();
        assert_error::<ZoneRequestError>();
        assert_error::<ApiError<'static>>();
        #[cfg(feature = "serde")]
        {
            assert_error::<ApiErrorResponse<'static>>();
            assert_error::<ResponseSizeError>();
            assert_error::<RrsetBodyError>();
        }
    }
}
