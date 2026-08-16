use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, BoundCredentialTransport,
    LocalAsyncAuthenticatedTransport,
};
use cloud_sdk::operation::{
    CheckedResponseGuard, ExecutionPermitError, PreparationStorageGuard, PreparedExecutionError,
    PreparedRequest, ResponsePolicyError,
};
use cloud_sdk::transport::{BoundTransport, ResponseBuffer};

use super::decode::{
    RobotOrderTransactionDecodeError, decode_addon, decode_addon_list, decode_market,
    decode_market_list, decode_standard, decode_standard_list,
};
use super::model::{
    RobotAddonTransaction, RobotAddonTransactionList, RobotMarketTransaction,
    RobotMarketTransactionList, RobotStandardTransaction, RobotStandardTransactionList,
};
use super::request::*;
use crate::robot::ordering::CredentialObserved;
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

    /// Executes and binds the checked response to one stable credential lifecycle.
    pub fn execute_observed_blocking<'buffer, T>(
        self,
        transport: &T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<
        CredentialCheckedRobotOrderTransaction<'buffer, 'request, R>,
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
        Ok(CredentialCheckedRobotOrderTransaction {
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
        CredentialCheckedRobotOrderTransaction<'buffer, 'request, R>,
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
        Ok(CredentialCheckedRobotOrderTransaction {
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
        CredentialCheckedRobotOrderTransaction<'buffer, 'request, R>,
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
        Ok(CredentialCheckedRobotOrderTransaction {
            request: self.request,
            inner,
            credential,
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

/// Checked transaction response carrying transport-established credential provenance.
pub struct CredentialCheckedRobotOrderTransaction<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
    credential: cloud_sdk::authentication::CredentialBinding,
}

impl<R> core::fmt::Debug for CredentialCheckedRobotOrderTransaction<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("CredentialCheckedRobotOrderTransaction([redacted])")
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
            pub fn prepare_bound<'guard, 'request>(
                &'request self,
                storage: &'guard mut PreparationStorageGuard<'_>,
            ) -> Result<PreparedRobotOrderTransaction<'guard, 'request, Self>, RobotOrderRequestError> {
                let inner = self.prepare_guarded(storage)?;
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

macro_rules! decode_observed_list {
    ($request:ty, $output:ty, $decoder:ident) => {
        impl CredentialCheckedRobotOrderTransaction<'_, '_, $request> {
            /// Decodes a transaction snapshot with credential provenance.
            pub fn decode_response(
                self,
            ) -> Result<CredentialObserved<$output>, RobotOrderTransactionDecodeError> {
                let value = self.inner.decode_owned_with_workspace($decoder)?;
                Ok(CredentialObserved::from_parts(value, self.credential))
            }
        }
    };
}

decode_observed_list!(
    RobotStandardTransactionListRequest,
    RobotStandardTransactionList,
    decode_standard_list
);
decode_observed_list!(
    RobotMarketTransactionListRequest,
    RobotMarketTransactionList,
    decode_market_list
);
decode_observed_list!(
    RobotAddonTransactionListRequest,
    RobotAddonTransactionList,
    decode_addon_list
);

macro_rules! decode_observed_item {
    ($request:ty, $output:ty, $decoder:ident) => {
        impl CredentialCheckedRobotOrderTransaction<'_, '_, $request> {
            /// Decodes the exact requested transaction with credential provenance.
            pub fn decode_response(
                self,
            ) -> Result<CredentialObserved<$output>, RobotOrderTransactionDecodeError> {
                let value = self.inner.decode_owned_with_workspace($decoder)?;
                if value.id() != &self.request.id {
                    return Err(RobotOrderTransactionDecodeError::ResponseIdentityMismatch);
                }
                Ok(CredentialObserved::from_parts(value, self.credential))
            }
        }
    };
}

decode_observed_item!(
    RobotStandardTransactionGetRequest,
    RobotStandardTransaction,
    decode_standard
);
decode_observed_item!(
    RobotMarketTransactionGetRequest,
    RobotMarketTransaction,
    decode_market
);
decode_observed_item!(
    RobotAddonTransactionGetRequest,
    RobotAddonTransaction,
    decode_addon
);
