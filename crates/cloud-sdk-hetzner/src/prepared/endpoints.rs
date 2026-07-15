//! Endpoint and existing request adapters.

use cloud_sdk::Method;
use cloud_sdk::operation::{CostIntent, PreparationStorage, PrepareOperation, PreparedRequest};

use crate::actions::{ActionEndpoint, ActionListRequest};
use crate::cloud::catalog::{
    CatalogGetEndpoint, CatalogListEndpoint, CatalogListRequest, CatalogSingletonEndpoint,
};
use crate::request::ApiBaseUrl;

use super::operation::method_metadata;
use super::{
    HetznerPreparationError, HetznerPreparedOperation, QueryWire, RequestShape, ResponseProfile,
};

impl crate::prepared::EndpointWire for ActionEndpoint {
    fn method(self) -> Method {
        self.method()
    }

    fn api_base_url(self) -> ApiBaseUrl {
        self.api_base_url()
    }

    fn write_path(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        self.write_path(output)
            .map_err(|_| HetznerPreparationError::Path)
    }

    fn request_shape(self) -> RequestShape {
        match self {
            Self::List => RequestShape::RequiredQuery,
            Self::Get(_) => RequestShape::None,
        }
    }

    fn response_profile(self) -> ResponseProfile {
        ResponseProfile::JsonOk
    }

    fn metadata(self) -> Result<cloud_sdk::operation::OperationMetadata, HetznerPreparationError> {
        method_metadata(self.method(), false, CostIntent::NoKnownCost)
    }

    fn operation_key(self) -> &'static str {
        match self {
            Self::List => "get_actions",
            Self::Get(_) => "get_action",
        }
    }
}

impl QueryWire for ActionListRequest<'_> {
    fn write_query(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        self.write_query(output)
            .map_err(|_| HetznerPreparationError::Query)
    }

    fn operation_key(self) -> &'static str {
        "get_actions"
    }
}

impl PrepareOperation for ActionEndpoint {
    type Error = HetznerPreparationError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        HetznerPreparedOperation::endpoint(*self).prepare(storage)
    }
}

impl PrepareOperation for ActionListRequest<'_> {
    type Error = HetznerPreparationError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        HetznerPreparedOperation::query(self.endpoint(), *self).prepare(storage)
    }
}

impl crate::prepared::EndpointWire for CatalogListEndpoint {
    fn method(self) -> Method {
        Method::Get
    }

    fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    fn write_path(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        let path = self.path().map_err(|_| HetznerPreparationError::Path)?;
        write_catalog_path(output, path.as_str())
    }

    fn request_shape(self) -> RequestShape {
        RequestShape::OptionalQuery
    }

    fn response_profile(self) -> ResponseProfile {
        ResponseProfile::JsonOk
    }

    fn metadata(self) -> Result<cloud_sdk::operation::OperationMetadata, HetznerPreparationError> {
        method_metadata(Method::Get, false, CostIntent::NoKnownCost)
    }

    fn operation_key(self) -> &'static str {
        match self {
            Self::Locations => "list_locations",
            Self::ServerTypes => "list_server_types",
            Self::LoadBalancerTypes => "list_load_balancer_types",
            Self::Isos => "list_isos",
            Self::PublicImages(_) => "list_images",
        }
    }
}

impl crate::prepared::EndpointWire for CatalogGetEndpoint {
    fn method(self) -> Method {
        Method::Get
    }

    fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    fn write_path(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        self.write_path(output)
            .map_err(|_| HetznerPreparationError::Path)
    }

    fn request_shape(self) -> RequestShape {
        RequestShape::None
    }

    fn response_profile(self) -> ResponseProfile {
        ResponseProfile::JsonOk
    }

    fn metadata(self) -> Result<cloud_sdk::operation::OperationMetadata, HetznerPreparationError> {
        method_metadata(Method::Get, false, CostIntent::NoKnownCost)
    }

    fn operation_key(self) -> &'static str {
        match self {
            Self::Location(_) => "get_location",
            Self::ServerType(_) => "get_server_type",
            Self::LoadBalancerType(_) => "get_load_balancer_type",
            Self::Iso(_) => "get_iso",
            Self::PublicImage(_) => "get_image",
        }
    }
}

impl crate::prepared::EndpointWire for CatalogSingletonEndpoint {
    fn method(self) -> Method {
        Method::Get
    }

    fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    fn write_path(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        let path = self.path().map_err(|_| HetznerPreparationError::Path)?;
        write_catalog_path(output, path.as_str())
    }

    fn request_shape(self) -> RequestShape {
        RequestShape::None
    }

    fn response_profile(self) -> ResponseProfile {
        ResponseProfile::JsonOk
    }

    fn metadata(self) -> Result<cloud_sdk::operation::OperationMetadata, HetznerPreparationError> {
        method_metadata(Method::Get, false, CostIntent::NoKnownCost)
    }

    fn operation_key(self) -> &'static str {
        "get_pricing"
    }
}

impl QueryWire for CatalogListRequest<'_> {
    fn write_query(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        self.write_query(output)
            .map_err(|_| HetznerPreparationError::Query)
    }

    fn operation_key(self) -> &'static str {
        crate::prepared::EndpointWire::operation_key(self.endpoint())
    }
}

macro_rules! impl_endpoint_prepare {
    ($($type:ty),+ $(,)?) => {$ (
        impl PrepareOperation for $type {
            type Error = HetznerPreparationError;

            fn prepare<'storage>(
                &self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRequest<'storage>, Self::Error> {
                HetznerPreparedOperation::endpoint(*self).prepare(storage)
            }
        }
    )+ };
}

impl_endpoint_prepare!(
    CatalogListEndpoint,
    CatalogGetEndpoint,
    CatalogSingletonEndpoint
);

impl PrepareOperation for CatalogListRequest<'_> {
    type Error = HetznerPreparationError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        HetznerPreparedOperation::query(self.endpoint(), *self).prepare(storage)
    }
}

fn write_catalog_path(output: &mut [u8], path: &str) -> Result<usize, HetznerPreparationError> {
    let target = output
        .get_mut(..path.len())
        .ok_or(HetznerPreparationError::Path)?;
    target.copy_from_slice(path.as_bytes());
    Ok(path.len())
}

macro_rules! endpoint_wire {
    (
        $type:ty,
        $value:ident => $shape:expr,
        $response:expr,
        match $key_value:ident {
            $($key_pattern:pat => $key:literal),+ $(,)?
        },
        $destructive:expr,
        $cost:expr
    ) => {
        impl crate::prepared::EndpointWire for $type {
            fn method(self) -> cloud_sdk::Method {
                <$type>::method(self)
            }

            fn api_base_url(self) -> crate::request::ApiBaseUrl {
                <$type>::api_base_url(self)
            }

            fn write_path(
                self,
                output: &mut [u8],
            ) -> Result<usize, super::HetznerPreparationError> {
                <$type>::write_path(self, output).map_err(|_| super::HetznerPreparationError::Path)
            }

            fn request_shape(self) -> super::RequestShape {
                let $value = self;
                let _ = $value;
                $shape
            }

            fn response_profile(self) -> super::ResponseProfile {
                let $value = self;
                let _ = $value;
                $response
            }

            fn metadata(
                self,
            ) -> Result<cloud_sdk::operation::OperationMetadata, super::HetznerPreparationError>
            {
                let $value = self;
                let _ = $value;
                crate::prepared::operation::method_metadata(self.method(), $destructive, $cost)
            }

            fn operation_key(self) -> &'static str {
                let $key_value = self;
                match $key_value {
                    $($key_pattern => $key),+
                }
            }
        }

        impl cloud_sdk::operation::PrepareOperation for $type {
            type Error = super::HetznerPreparationError;

            fn prepare<'storage>(
                &self,
                storage: cloud_sdk::operation::PreparationStorage<'storage>,
            ) -> Result<cloud_sdk::operation::PreparedRequest<'storage>, Self::Error> {
                super::HetznerPreparedOperation::endpoint(*self).prepare(storage)
            }
        }
    };
}

macro_rules! query_wire {
    ($type:ty, $value:ident => $endpoint:expr) => {
        impl super::QueryWire for $type {
            fn write_query(
                self,
                output: &mut [u8],
            ) -> Result<usize, super::HetznerPreparationError> {
                <$type>::write_query(self, output)
                    .map_err(|_| super::HetznerPreparationError::Query)
            }

            fn operation_key(self) -> &'static str {
                let $value = self;
                crate::prepared::EndpointWire::operation_key($endpoint)
            }
        }

        impl cloud_sdk::operation::PrepareOperation for $type {
            type Error = super::HetznerPreparationError;

            fn prepare<'storage>(
                &self,
                storage: cloud_sdk::operation::PreparationStorage<'storage>,
            ) -> Result<cloud_sdk::operation::PreparedRequest<'storage>, Self::Error> {
                let $value = *self;
                super::HetznerPreparedOperation::query($endpoint, *self).prepare(storage)
            }
        }
    };
}

mod compute;
mod network;
mod security_dns;
mod storage;
