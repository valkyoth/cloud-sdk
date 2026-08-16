use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorageGuard, PreparedRequest, ResponsePolicyError,
};
use cloud_sdk::transport::ResponseBuffer;

use super::prepare::{Kind, prepare};
use super::request::*;
use crate::robot::ordering::transaction::{
    decode_addon_created, decode_market_created, decode_standard,
};
use crate::robot::ordering::{
    RobotAddonTransaction, RobotMarketCreatedTransaction, RobotStandardTransaction,
};

/// Guarded prepared billable order retaining its exact catalog-derived intent.
pub struct PreparedRobotOrderMutation<'storage, 'request, R> {
    pub(super) request: &'request R,
    pub(super) inner: PreparedRequest<'storage>,
}

impl<'storage, 'request, R> PreparedRobotOrderMutation<'storage, 'request, R> {
    /// Applies exact `201` response policy while retaining order provenance.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedRobotOrderMutation<'buffer, 'request, R>, ResponsePolicyError> {
        self.inner
            .validate_response(response)
            .map(|inner| CheckedRobotOrderMutation {
                request: self.request,
                inner,
            })
    }

    pub(super) fn into_plan_parts(self) -> (PreparedRequest<'storage>, &'request R) {
        (self.inner, self.request)
    }
}

impl<R> core::fmt::Debug for PreparedRobotOrderMutation<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("PreparedRobotOrderMutation([redacted])")
    }
}

/// Checked successful order response retaining its exact confirmed request.
pub struct CheckedRobotOrderMutation<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
}

impl<'buffer, 'request, R> CheckedRobotOrderMutation<'buffer, 'request, R> {
    pub(super) const fn from_executed(
        request: &'request R,
        inner: CheckedResponseGuard<'buffer>,
    ) -> Self {
        Self { request, inner }
    }
}

impl<R> core::fmt::Debug for CheckedRobotOrderMutation<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("CheckedRobotOrderMutation([redacted])")
    }
}

macro_rules! prepare_bound {
    ($type:ty, $kind:ident) => {
        impl $type {
            /// Prepares this billable operation only behind cleanup-owned storage.
            pub fn prepare_bound<'guard, 'request>(
                &'request self,
                storage: &'guard mut PreparationStorageGuard<'_>,
            ) -> Result<
                PreparedRobotOrderMutation<'guard, 'request, Self>,
                RobotOrderMutationRequestError,
            > {
                storage.prepare_with(|storage| {
                    prepare(Kind::$kind(self), storage).map(|inner| PreparedRobotOrderMutation {
                        request: self,
                        inner,
                    })
                })
            }
        }
    };
}

prepare_bound!(RobotStandardOrderCreateRequest<'_>, Standard);
prepare_bound!(RobotMarketOrderCreateRequest<'_>, Market);
prepare_bound!(RobotAddonOrderCreateRequest<'_, '_>, Addon);

impl CheckedRobotOrderMutation<'_, '_, RobotStandardOrderCreateRequest<'_>> {
    /// Decodes the created transaction and verifies the complete observable intent.
    pub fn decode_response(
        self,
    ) -> Result<RobotStandardTransaction, RobotOrderMutationDecodeError> {
        let value = self
            .inner
            .decode_owned_with_workspace(decode_standard)
            .map_err(RobotOrderMutationDecodeError::Transaction)?;
        self.request
            .matches_transaction(&value)
            .then_some(value)
            .ok_or(RobotOrderMutationDecodeError::ResponseIntentMismatch)
    }
}

impl CheckedRobotOrderMutation<'_, '_, RobotMarketOrderCreateRequest<'_>> {
    /// Decodes the created transaction and verifies the complete observable intent.
    pub fn decode_response(
        self,
    ) -> Result<RobotMarketCreatedTransaction, RobotOrderMutationDecodeError> {
        let value = self
            .inner
            .decode_owned_with_workspace(decode_market_created)
            .map_err(RobotOrderMutationDecodeError::Transaction)?;
        self.request
            .matches_created_transaction(&value)
            .then_some(value)
            .ok_or(RobotOrderMutationDecodeError::ResponseIntentMismatch)
    }
}

impl CheckedRobotOrderMutation<'_, '_, RobotAddonOrderCreateRequest<'_, '_>> {
    /// Decodes the created transaction and verifies server and product identity.
    pub fn decode_response(self) -> Result<RobotAddonTransaction, RobotOrderMutationDecodeError> {
        let value = self
            .inner
            .decode_owned_with_workspace(decode_addon_created)
            .map_err(RobotOrderMutationDecodeError::Transaction)?;
        self.request
            .matches_created_transaction(&value)
            .then_some(value)
            .ok_or(RobotOrderMutationDecodeError::ResponseIntentMismatch)
    }
}
