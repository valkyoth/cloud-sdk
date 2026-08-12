use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PrepareOperation, PreparedRequest,
    ResponsePolicyError,
};
use cloud_sdk::transport::ResponseBuffer;

use super::decode::{
    RobotResetDecodeError, decode_robot_reset, decode_robot_reset_action, decode_robot_reset_list,
};
use super::model::{RobotReset, RobotResetAction, RobotResetList};
use super::{
    RobotResetExecuteRequest, RobotResetGetRequest, RobotResetListRequest, RobotResetRequestError,
};

/// Prepared Robot reset request retaining its exact typed association.
pub struct PreparedRobotReset<'storage, 'request, R> {
    pub(super) request: &'request R,
    pub(super) inner: PreparedRequest<'storage>,
}

impl<'storage, 'request, R> PreparedRobotReset<'storage, 'request, R> {
    /// Applies the exact response policy and retains request provenance.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedRobotReset<'buffer, 'request, R>, ResponsePolicyError> {
        self.inner
            .validate_response(response)
            .map(|inner| CheckedRobotReset {
                request: self.request,
                inner,
            })
    }

    pub(super) fn into_plan_parts(self) -> (PreparedRequest<'storage>, &'request R) {
        (self.inner, self.request)
    }
}

impl<'storage, 'request> PreparedRobotReset<'storage, 'request, RobotResetListRequest> {
    /// Returns the provider-neutral read request for generic execution.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'storage> {
        self.inner
    }
}

impl<'storage, 'request> PreparedRobotReset<'storage, 'request, RobotResetGetRequest> {
    /// Returns the provider-neutral read request for generic execution.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'storage> {
        self.inner
    }
}

impl<R> core::fmt::Debug for PreparedRobotReset<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PreparedRobotReset")
            .field("request", &"[bound]")
            .field("prepared", &self.inner)
            .finish()
    }
}

/// Checked Robot reset response retaining the exact admitting request.
pub struct CheckedRobotReset<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
}

impl<'buffer, 'request, R> CheckedRobotReset<'buffer, 'request, R> {
    pub(super) const fn from_executed(
        request: &'request R,
        inner: CheckedResponseGuard<'buffer>,
    ) -> Self {
        Self { request, inner }
    }
}

impl<R> core::fmt::Debug for CheckedRobotReset<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CheckedRobotReset")
            .field("request", &"[bound]")
            .field("response", &"[checked]")
            .finish()
    }
}

macro_rules! prepare_bound {
    ($($type:ty),+ $(,)?) => {$ (
        impl $type {
            /// Prepares this operation while retaining exact response association.
            pub fn prepare_bound<'storage, 'request>(
                &'request self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRobotReset<'storage, 'request, Self>, RobotResetRequestError> {
                let inner = self.prepare(storage)?;
                Ok(PreparedRobotReset { request: self, inner })
            }
        }
    )+ };
}

prepare_bound!(RobotResetListRequest, RobotResetGetRequest);

impl CheckedRobotReset<'_, '_, RobotResetListRequest> {
    /// Decodes a bounded duplicate-free reset capability list.
    pub fn decode_response(self) -> Result<RobotResetList, RobotResetDecodeError> {
        self.inner
            .decode_owned_with_workspace(decode_robot_reset_list)
    }
}

impl CheckedRobotReset<'_, '_, RobotResetGetRequest> {
    /// Decodes and binds reset state to the requested server number.
    pub fn decode_response(self) -> Result<RobotReset, RobotResetDecodeError> {
        self.inner
            .decode_owned_with_workspace(|response, workspace| {
                decode_robot_reset(response, self.request.number(), workspace)
            })
    }
}

impl CheckedRobotReset<'_, '_, RobotResetExecuteRequest<'_>> {
    /// Decodes and binds an action acknowledgement to checked preflight state.
    pub fn decode_response(self) -> Result<RobotResetAction, RobotResetDecodeError> {
        let action = self
            .inner
            .decode_owned_with_workspace(decode_robot_reset_action)?;
        let expected = self.request.reset.reset().summary();
        let ipv4_matches = action.server_ipv4 == expected.server_ipv4;
        let ipv6_matches = action.server_ipv6_network == expected.server_ipv6_network;
        let number_matches = action
            .number
            .as_ref()
            .is_none_or(|number| number == expected.number());
        if !ipv4_matches || !ipv6_matches || !number_matches {
            return Err(RobotResetDecodeError::ResponseIdentityMismatch);
        }
        if action.reset_type != self.request.intent.reset_type() {
            return Err(RobotResetDecodeError::MutationOutcomeMismatch);
        }
        Ok(action)
    }
}

#[cfg(doctest)]
mod compile_fail {
    /// A list response cannot be decoded as a reset action.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::{CheckedRobotReset, RobotResetListRequest, RobotResetAction};
    /// fn wrong(response: CheckedRobotReset<'_, '_, RobotResetListRequest>) {
    ///     let _: RobotResetAction = response.decode_response().unwrap();
    /// }
    /// ```
    fn association() {}

    /// Reset execution cannot enter generic operation preparation.
    ///
    /// ```compile_fail
    /// use cloud_sdk::operation::PrepareOperation;
    /// use cloud_sdk_hetzner::robot::RobotResetExecuteRequest;
    /// fn generic<R: PrepareOperation>() {}
    /// generic::<RobotResetExecuteRequest<'static>>();
    /// ```
    fn generic_preparation() {}

    /// Prepared reset execution cannot erase its typed association.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::{PreparedRobotReset, RobotResetExecuteRequest};
    /// fn erase(request: PreparedRobotReset<'_, '_, RobotResetExecuteRequest<'_>>) {
    ///     let _ = request.as_untyped();
    /// }
    /// ```
    fn execute_type_erasure() {}
}
