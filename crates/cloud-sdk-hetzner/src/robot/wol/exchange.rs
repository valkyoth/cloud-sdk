use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PrepareOperation, PreparedRequest,
    ResponsePolicyError,
};
use cloud_sdk::transport::ResponseBuffer;

use super::{
    RobotWol, RobotWolDecodeError, RobotWolGetRequest, RobotWolRequestError, RobotWolSendRequest,
    decode_robot_wol,
};

/// Prepared WOL request retaining exact request association.
pub struct PreparedRobotWol<'storage, 'request, R> {
    pub(super) request: &'request R,
    pub(super) inner: PreparedRequest<'storage>,
}

impl<'storage, 'request, R> PreparedRobotWol<'storage, 'request, R> {
    /// Applies exact response policy while retaining request provenance.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedRobotWol<'buffer, 'request, R>, ResponsePolicyError> {
        self.inner
            .validate_response(response)
            .map(|inner| CheckedRobotWol {
                request: self.request,
                inner,
            })
    }

    pub(super) fn into_plan_parts(self) -> (PreparedRequest<'storage>, &'request R) {
        (self.inner, self.request)
    }
}

impl PreparedRobotWol<'_, '_, RobotWolGetRequest> {
    /// Returns the provider-neutral read request for generic execution.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'_> {
        self.inner
    }
}

impl<R> core::fmt::Debug for PreparedRobotWol<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PreparedRobotWol")
            .field("request", &"[bound]")
            .field("prepared", &self.inner)
            .finish()
    }
}

/// Checked WOL response retaining the request that admitted it.
pub struct CheckedRobotWol<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
}

impl<'buffer, 'request, R> CheckedRobotWol<'buffer, 'request, R> {
    pub(super) const fn from_executed(
        request: &'request R,
        inner: CheckedResponseGuard<'buffer>,
    ) -> Self {
        Self { request, inner }
    }
}

impl<R> core::fmt::Debug for CheckedRobotWol<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("CheckedRobotWol([redacted])")
    }
}

impl RobotWolGetRequest {
    /// Prepares discovery while retaining exact response association.
    pub fn prepare_bound<'storage, 'request>(
        &'request self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRobotWol<'storage, 'request, Self>, RobotWolRequestError> {
        let inner = self.prepare(storage)?;
        Ok(PreparedRobotWol {
            request: self,
            inner,
        })
    }
}

impl CheckedRobotWol<'_, '_, RobotWolGetRequest> {
    /// Decodes and binds discovered capability to the requested server.
    pub fn decode_response(self) -> Result<RobotWol, RobotWolDecodeError> {
        self.inner
            .decode_owned_with_workspace(|response, workspace| {
                decode_robot_wol(response, self.request.number(), workspace)
            })
    }
}

impl CheckedRobotWol<'_, '_, RobotWolSendRequest<'_>> {
    /// Requires the wake acknowledgement to retain exact checked identity.
    pub fn decode_response(self) -> Result<RobotWol, RobotWolDecodeError> {
        let expected = self.request.wol.wol();
        let actual = self
            .inner
            .decode_owned_with_workspace(|response, workspace| {
                decode_robot_wol(response, self.request.number(), workspace)
            })?;
        if !actual.same_identity(expected) {
            return Err(RobotWolDecodeError::ResponseIdentityMismatch);
        }
        Ok(actual)
    }
}

#[cfg(doctest)]
mod compile_fail {
    /// WOL execution cannot enter generic preparation without capability evidence.
    ///
    /// ```compile_fail
    /// use cloud_sdk::operation::PrepareOperation;
    /// use cloud_sdk_hetzner::robot::RobotWolSendRequest;
    /// fn generic<R: PrepareOperation>() {}
    /// generic::<RobotWolSendRequest<'static>>();
    /// ```
    fn generic_preparation() {}

    /// Prepared WOL execution cannot erase its typed association.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::{PreparedRobotWol, RobotWolSendRequest};
    /// fn erase(request: PreparedRobotWol<'_, '_, RobotWolSendRequest<'_>>) {
    ///     let _ = request.as_untyped();
    /// }
    /// ```
    fn execution_type_erasure() {}
}
