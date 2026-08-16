use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, BoundCredentialTransport,
    LocalAsyncAuthenticatedTransport,
};
use cloud_sdk::operation::{
    CheckedResponseGuard, ExecutionPermitError, PreparationStorage, PrepareOperation,
    PreparedExecutionError, PreparedRequest, ResponsePolicyError,
};
use cloud_sdk::transport::{BoundTransport, ResponseBuffer};

use super::CredentialObserved;
use super::RobotOrderCurrency;
use super::decode::{
    RobotOrderCatalogDecodeError, decode_addon_list, decode_currency, decode_market,
    decode_market_list, decode_standard, decode_standard_list,
};
use super::model::{
    RobotAddonProductList, RobotMarketProduct, RobotMarketProductList, RobotStandardProduct,
    RobotStandardProductList,
};
use super::request::*;
use crate::robot::RobotServerNumber;

/// Prepared Robot ordering-catalog request retaining exact request association.
pub struct PreparedRobotOrderCatalog<'storage, 'request, R> {
    request: &'request R,
    inner: PreparedRequest<'storage>,
}

impl<'storage, 'request, R> PreparedRobotOrderCatalog<'storage, 'request, R> {
    /// Borrows the provider-neutral prepared request for inspection.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'storage> {
        self.inner
    }

    /// Applies the exact response policy and retains request provenance.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedRobotOrderCatalog<'buffer, 'request, R>, ResponsePolicyError> {
        self.inner
            .validate_response(response)
            .map(|inner| CheckedRobotOrderCatalog {
                request: self.request,
                inner,
            })
    }

    /// Executes and binds the checked response to one stable credential lifecycle.
    pub fn execute_observed_blocking<'buffer, T>(
        self,
        transport: &T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<
        CredentialCheckedRobotOrderCatalog<'buffer, 'request, R>,
        PreparedExecutionError<T::Error>,
    >
    where
        T: BlockingAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
    {
        let credential = transport.credential_binding();
        let request = self.request;
        let inner =
            self.inner
                .execute_blocking(transport, response_storage, response_header_storage)?;
        require_stable_credential::<T::Error>(credential, transport.credential_binding())?;
        Ok(CredentialCheckedRobotOrderCatalog {
            request,
            inner,
            credential,
        })
    }

    /// Send-async equivalent of [`Self::execute_observed_blocking`].
    pub async fn execute_observed_async<'transport, 'buffer, T>(
        &'transport self,
        transport: &'transport T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<
        CredentialCheckedRobotOrderCatalog<'buffer, 'request, R>,
        PreparedExecutionError<T::Error>,
    >
    where
        T: AsyncAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
        'storage: 'transport,
    {
        let credential = transport.credential_binding();
        let inner = self
            .inner
            .execute_async(transport, response_storage, response_header_storage)
            .await?;
        require_stable_credential::<T::Error>(credential, transport.credential_binding())?;
        Ok(CredentialCheckedRobotOrderCatalog {
            request: self.request,
            inner,
            credential,
        })
    }

    /// Local-async equivalent of [`Self::execute_observed_blocking`].
    pub async fn execute_observed_local_async<'transport, 'buffer, T>(
        &'transport self,
        transport: &'transport T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<
        CredentialCheckedRobotOrderCatalog<'buffer, 'request, R>,
        PreparedExecutionError<T::Error>,
    >
    where
        T: LocalAsyncAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
        'storage: 'transport,
    {
        let credential = transport.credential_binding();
        let inner = self
            .inner
            .execute_local_async(transport, response_storage, response_header_storage)
            .await?;
        require_stable_credential::<T::Error>(credential, transport.credential_binding())?;
        Ok(CredentialCheckedRobotOrderCatalog {
            request: self.request,
            inner,
            credential,
        })
    }
}

impl<R> core::fmt::Debug for PreparedRobotOrderCatalog<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PreparedRobotOrderCatalog")
            .field("request", &"[bound]")
            .field("prepared", &self.inner)
            .finish()
    }
}

/// Checked Robot ordering-catalog response retaining its admitting request.
pub struct CheckedRobotOrderCatalog<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
}

/// Checked catalog response carrying transport-established credential provenance.
pub struct CredentialCheckedRobotOrderCatalog<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
    credential: cloud_sdk::authentication::CredentialBinding,
}

impl<R> core::fmt::Debug for CredentialCheckedRobotOrderCatalog<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("CredentialCheckedRobotOrderCatalog([redacted])")
    }
}

fn require_stable_credential<E>(
    before: cloud_sdk::authentication::CredentialBinding,
    after: cloud_sdk::authentication::CredentialBinding,
) -> Result<(), PreparedExecutionError<E>> {
    if before.matches(after) {
        Ok(())
    } else {
        Err(PreparedExecutionError::AuthorizationInvalid(
            ExecutionPermitError::CredentialMismatch,
        ))
    }
}

/// Per-server addon catalog retaining the exact request that admitted it.
pub struct RobotAddonCatalog<'request> {
    request: &'request RobotAddonProductListRequest,
    products: RobotAddonProductList,
}

impl RobotAddonCatalog<'_> {
    /// Returns the server whose addon catalog was requested.
    #[must_use]
    pub const fn server(&self) -> &RobotServerNumber {
        &self.request.server
    }

    /// Returns the bounded products advertised for that server.
    #[must_use]
    pub const fn products(&self) -> &RobotAddonProductList {
        &self.products
    }
}

impl core::fmt::Debug for RobotAddonCatalog<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotAddonCatalog([redacted])")
    }
}

impl<R> core::fmt::Debug for CheckedRobotOrderCatalog<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CheckedRobotOrderCatalog")
            .field("request", &"[bound]")
            .field("response", &"[checked]")
            .finish()
    }
}

macro_rules! prepare_bound {
    ($($type:ty),+ $(,)?) => {$ (
        impl $type {
            /// Prepares this read-only operation with exact response association.
            pub fn prepare_bound<'storage, 'request>(
                &'request self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRobotOrderCatalog<'storage, 'request, Self>, RobotOrderRequestError> {
                let inner = self.prepare(storage)?;
                Ok(PreparedRobotOrderCatalog { request: self, inner })
            }
        }
    )+ };
}

prepare_bound!(
    RobotStandardProductListRequest,
    RobotStandardProductGetRequest,
    RobotMarketProductListRequest,
    RobotMarketProductGetRequest,
    RobotAddonProductListRequest,
    RobotOrderCurrencyRequest,
);

impl CheckedRobotOrderCatalog<'_, '_, RobotStandardProductListRequest> {
    /// Decodes the bounded standard-server product catalog.
    pub fn decode_response(self) -> Result<RobotStandardProductList, RobotOrderCatalogDecodeError> {
        self.inner.decode_owned_with_workspace(decode_standard_list)
    }
}

impl CheckedRobotOrderCatalog<'_, '_, RobotStandardProductGetRequest> {
    /// Decodes one product and requires the exact requested identity.
    pub fn decode_response(self) -> Result<RobotStandardProduct, RobotOrderCatalogDecodeError> {
        let product = self.inner.decode_owned_with_workspace(decode_standard)?;
        if product.id() == &self.request.id {
            Ok(product)
        } else {
            Err(RobotOrderCatalogDecodeError::ResponseIdentityMismatch)
        }
    }
}

impl CheckedRobotOrderCatalog<'_, '_, RobotMarketProductListRequest> {
    /// Decodes the bounded Server Auction catalog.
    pub fn decode_response(self) -> Result<RobotMarketProductList, RobotOrderCatalogDecodeError> {
        self.inner.decode_owned_with_workspace(decode_market_list)
    }
}

impl CheckedRobotOrderCatalog<'_, '_, RobotMarketProductGetRequest> {
    /// Decodes one Server Auction product and requires its requested identity.
    pub fn decode_response(self) -> Result<RobotMarketProduct, RobotOrderCatalogDecodeError> {
        let product = self.inner.decode_owned_with_workspace(decode_market)?;
        if product.id() == self.request.id {
            Ok(product)
        } else {
            Err(RobotOrderCatalogDecodeError::ResponseIdentityMismatch)
        }
    }
}

impl<'request> CheckedRobotOrderCatalog<'_, 'request, RobotAddonProductListRequest> {
    /// Decodes addons currently advertised for the request-bound server.
    pub fn decode_response(
        self,
    ) -> Result<RobotAddonCatalog<'request>, RobotOrderCatalogDecodeError> {
        let products = self.inner.decode_owned_with_workspace(decode_addon_list)?;
        Ok(RobotAddonCatalog {
            request: self.request,
            products,
        })
    }
}

impl CheckedRobotOrderCatalog<'_, '_, RobotOrderCurrencyRequest> {
    /// Decodes the authenticated account currency.
    pub fn decode_response(self) -> Result<RobotOrderCurrency, RobotOrderCatalogDecodeError> {
        self.inner.decode_owned_with_workspace(decode_currency)
    }
}

macro_rules! decode_observed_catalog {
    ($request:ty, $output:ty, $decoder:ident) => {
        impl CredentialCheckedRobotOrderCatalog<'_, '_, $request> {
            /// Decodes a value while retaining transport-established credential provenance.
            pub fn decode_response(
                self,
            ) -> Result<CredentialObserved<$output>, RobotOrderCatalogDecodeError> {
                let value = self.inner.decode_owned_with_workspace($decoder)?;
                Ok(CredentialObserved::from_parts(value, self.credential))
            }
        }
    };
}

decode_observed_catalog!(
    RobotStandardProductListRequest,
    RobotStandardProductList,
    decode_standard_list
);
decode_observed_catalog!(
    RobotMarketProductListRequest,
    RobotMarketProductList,
    decode_market_list
);
decode_observed_catalog!(
    RobotOrderCurrencyRequest,
    RobotOrderCurrency,
    decode_currency
);

impl CredentialCheckedRobotOrderCatalog<'_, '_, RobotStandardProductGetRequest> {
    /// Decodes the exact requested product with credential provenance.
    pub fn decode_response(
        self,
    ) -> Result<CredentialObserved<RobotStandardProduct>, RobotOrderCatalogDecodeError> {
        let product = self.inner.decode_owned_with_workspace(decode_standard)?;
        if product.id() != &self.request.id {
            return Err(RobotOrderCatalogDecodeError::ResponseIdentityMismatch);
        }
        Ok(CredentialObserved::from_parts(product, self.credential))
    }
}

impl CredentialCheckedRobotOrderCatalog<'_, '_, RobotMarketProductGetRequest> {
    /// Decodes the exact requested auction product with credential provenance.
    pub fn decode_response(
        self,
    ) -> Result<CredentialObserved<RobotMarketProduct>, RobotOrderCatalogDecodeError> {
        let product = self.inner.decode_owned_with_workspace(decode_market)?;
        if product.id() != self.request.id {
            return Err(RobotOrderCatalogDecodeError::ResponseIdentityMismatch);
        }
        Ok(CredentialObserved::from_parts(product, self.credential))
    }
}

impl<'request> CredentialCheckedRobotOrderCatalog<'_, 'request, RobotAddonProductListRequest> {
    /// Decodes the request-bound addon catalog with credential provenance.
    pub fn decode_response(
        self,
    ) -> Result<CredentialObserved<RobotAddonCatalog<'request>>, RobotOrderCatalogDecodeError> {
        let products = self.inner.decode_owned_with_workspace(decode_addon_list)?;
        Ok(CredentialObserved::from_parts(
            RobotAddonCatalog {
                request: self.request,
                products,
            },
            self.credential,
        ))
    }
}

#[cfg(doctest)]
mod compile_fail {
    /// Different catalog operations cannot exchange checked responses.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::{
    ///     CheckedRobotOrderCatalog, RobotMarketProductListRequest,
    ///     RobotStandardProductListRequest,
    /// };
    /// fn consume(_: CheckedRobotOrderCatalog<'_, '_, RobotMarketProductListRequest>) {}
    /// fn wrong(response: CheckedRobotOrderCatalog<'_, '_, RobotStandardProductListRequest>) {
    ///     consume(response);
    /// }
    /// ```
    fn association() {}
}
