use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PrepareOperation, PreparedRequest,
    ResponsePolicyError,
};
use cloud_sdk::transport::ResponseBuffer;

use super::decode::decode_robot_traffic;
use super::{
    RobotTrafficDecodeError, RobotTrafficReport, RobotTrafficRequest, RobotTrafficRequestError,
};

/// Prepared traffic request retaining its exact typed association.
pub struct PreparedRobotTraffic<'storage, 'request> {
    request: &'request RobotTrafficRequest,
    inner: PreparedRequest<'storage>,
}

impl<'storage, 'request> PreparedRobotTraffic<'storage, 'request> {
    /// Borrows the provider-neutral prepared request.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'storage> {
        self.inner
    }

    /// Applies the exact response policy and retains request provenance.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedRobotTraffic<'buffer, 'request>, ResponsePolicyError> {
        self.inner
            .validate_response(response)
            .map(|inner| CheckedRobotTraffic {
                request: self.request,
                inner,
            })
    }
}

impl core::fmt::Debug for PreparedRobotTraffic<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("PreparedRobotTraffic([bound])")
    }
}

/// Checked traffic response retaining its exact admitting request.
pub struct CheckedRobotTraffic<'buffer, 'request> {
    request: &'request RobotTrafficRequest,
    inner: CheckedResponseGuard<'buffer>,
}

impl CheckedRobotTraffic<'_, '_> {
    /// Incrementally decodes and binds the report to the complete request.
    pub fn decode_response(self) -> Result<RobotTrafficReport, RobotTrafficDecodeError> {
        let request = self.request;
        self.inner
            .decode_owned_with_workspace(|response, _| decode_robot_traffic(response, request))
    }
}

impl core::fmt::Debug for CheckedRobotTraffic<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("CheckedRobotTraffic([bound, checked])")
    }
}

impl RobotTrafficRequest {
    /// Prepares this query while retaining exact response association.
    pub fn prepare_bound<'storage, 'request>(
        &'request self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRobotTraffic<'storage, 'request>, RobotTrafficRequestError> {
        self.prepare(storage).map(|inner| PreparedRobotTraffic {
            request: self,
            inner,
        })
    }
}
