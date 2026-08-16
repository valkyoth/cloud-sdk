use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PrepareOperation, PreparedRequest,
    ResponsePolicyError,
};
use cloud_sdk::transport::ResponseBuffer;

use super::decode::{
    RobotOrderTransactionDecodeError, decode_addon, decode_addon_list, decode_market,
    decode_market_list, decode_standard, decode_standard_list,
};
use super::model::{
    RobotAddonTransaction, RobotAddonTransactionList, RobotMarketTransaction,
    RobotMarketTransactionList, RobotStandardTransaction, RobotStandardTransactionList,
};
use super::request::*;
use crate::robot::ordering::RobotOrderRequestError;

/// Prepared read-only Robot transaction request retaining exact association.
pub struct PreparedRobotOrderTransaction<'storage, 'request, R> {
    request: &'request R,
    inner: PreparedRequest<'storage>,
}

impl<'storage, 'request, R> PreparedRobotOrderTransaction<'storage, 'request, R> {
    /// Borrows the provider-neutral prepared request for inspection.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'storage> {
        self.inner
    }

    /// Applies the exact response policy and retains request provenance.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedRobotOrderTransaction<'buffer, 'request, R>, ResponsePolicyError> {
        self.inner
            .validate_response(response)
            .map(|inner| CheckedRobotOrderTransaction {
                request: self.request,
                inner,
            })
    }
}

impl<R> core::fmt::Debug for PreparedRobotOrderTransaction<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PreparedRobotOrderTransaction")
            .field("request", &"[bound]")
            .field("prepared", &self.inner)
            .finish()
    }
}

/// Checked Robot transaction response retaining its admitting request.
pub struct CheckedRobotOrderTransaction<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
}

impl<R> core::fmt::Debug for CheckedRobotOrderTransaction<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CheckedRobotOrderTransaction")
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
            ) -> Result<PreparedRobotOrderTransaction<'storage, 'request, Self>, RobotOrderRequestError> {
                let inner = self.prepare(storage)?;
                Ok(PreparedRobotOrderTransaction { request: self, inner })
            }
        }
    )+ };
}

prepare_bound!(
    RobotStandardTransactionListRequest,
    RobotStandardTransactionGetRequest,
    RobotMarketTransactionListRequest,
    RobotMarketTransactionGetRequest,
    RobotAddonTransactionListRequest,
    RobotAddonTransactionGetRequest,
);

impl CheckedRobotOrderTransaction<'_, '_, RobotStandardTransactionListRequest> {
    /// Decodes the bounded standard-server transaction snapshot.
    pub fn decode_response(
        self,
    ) -> Result<RobotStandardTransactionList, RobotOrderTransactionDecodeError> {
        self.inner.decode_owned_with_workspace(decode_standard_list)
    }
}

impl CheckedRobotOrderTransaction<'_, '_, RobotStandardTransactionGetRequest> {
    /// Decodes one standard-server transaction and binds its exact identity.
    pub fn decode_response(
        self,
    ) -> Result<RobotStandardTransaction, RobotOrderTransactionDecodeError> {
        let transaction = self.inner.decode_owned_with_workspace(decode_standard)?;
        if transaction.id() == &self.request.id {
            Ok(transaction)
        } else {
            Err(RobotOrderTransactionDecodeError::ResponseIdentityMismatch)
        }
    }
}

impl CheckedRobotOrderTransaction<'_, '_, RobotMarketTransactionListRequest> {
    /// Decodes the bounded Server Auction transaction snapshot.
    pub fn decode_response(
        self,
    ) -> Result<RobotMarketTransactionList, RobotOrderTransactionDecodeError> {
        self.inner.decode_owned_with_workspace(decode_market_list)
    }
}

impl CheckedRobotOrderTransaction<'_, '_, RobotMarketTransactionGetRequest> {
    /// Decodes one Server Auction transaction and binds its exact identity.
    pub fn decode_response(
        self,
    ) -> Result<RobotMarketTransaction, RobotOrderTransactionDecodeError> {
        let transaction = self.inner.decode_owned_with_workspace(decode_market)?;
        if transaction.id() == &self.request.id {
            Ok(transaction)
        } else {
            Err(RobotOrderTransactionDecodeError::ResponseIdentityMismatch)
        }
    }
}

impl CheckedRobotOrderTransaction<'_, '_, RobotAddonTransactionListRequest> {
    /// Decodes the bounded addon transaction snapshot.
    pub fn decode_response(
        self,
    ) -> Result<RobotAddonTransactionList, RobotOrderTransactionDecodeError> {
        self.inner.decode_owned_with_workspace(decode_addon_list)
    }
}

impl CheckedRobotOrderTransaction<'_, '_, RobotAddonTransactionGetRequest> {
    /// Decodes one addon transaction and binds its exact identity.
    pub fn decode_response(
        self,
    ) -> Result<RobotAddonTransaction, RobotOrderTransactionDecodeError> {
        let transaction = self.inner.decode_owned_with_workspace(decode_addon)?;
        if transaction.id() == &self.request.id {
            Ok(transaction)
        } else {
            Err(RobotOrderTransactionDecodeError::ResponseIdentityMismatch)
        }
    }
}
