use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PrepareOperation, PreparedRequest,
    ResponsePolicyError,
};
use cloud_sdk::transport::ResponseBuffer;

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
