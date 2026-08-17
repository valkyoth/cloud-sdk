use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PrepareOperation, PreparedRequest,
    ResponsePolicyError,
};
use cloud_sdk::transport::ResponseBuffer;

use super::decode::{RobotFailoverDecodeError, decode_robot_failover, decode_robot_failover_list};
use super::model::{RobotFailover, RobotFailoverList};
use super::request::*;

/// Prepared Robot failover request retaining its exact typed association.
pub struct PreparedRobotFailover<'storage, 'request, R> {
    request: &'request R,
    inner: PreparedRequest<'storage>,
}

impl<'storage, 'request, R> PreparedRobotFailover<'storage, 'request, R> {
    /// Borrows the provider-neutral prepared request for inspection.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'storage> {
        self.inner
    }

    /// Applies the exact response policy and retains request provenance.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedRobotFailover<'buffer, 'request, R>, ResponsePolicyError> {
        self.inner
            .validate_response(response)
            .map(|inner| CheckedRobotFailover {
                request: self.request,
                inner,
            })
    }

    pub(super) fn into_plan_parts(self) -> (PreparedRequest<'storage>, &'request R) {
        (self.inner, self.request)
    }
}

impl<R> core::fmt::Debug for PreparedRobotFailover<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PreparedRobotFailover")
            .field("request", &"[bound]")
            .field("prepared", &self.inner)
            .finish()
    }
}

/// Checked Robot failover response retaining the request that admitted it.
pub struct CheckedRobotFailover<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
}

impl<'buffer, 'request, R> CheckedRobotFailover<'buffer, 'request, R> {
    pub(crate) const fn from_executed(
        request: &'request R,
        inner: CheckedResponseGuard<'buffer>,
    ) -> Self {
        Self { request, inner }
    }
}

impl<R> core::fmt::Debug for CheckedRobotFailover<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CheckedRobotFailover")
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
            ) -> Result<PreparedRobotFailover<'storage, 'request, Self>, RobotFailoverRequestError> {
                let inner = self.prepare(storage)?;
                Ok(PreparedRobotFailover { request: self, inner })
            }
        }
    )+ };
}

prepare_bound!(
    RobotFailoverListRequest,
    RobotFailoverGetRequest,
    RobotFailoverRerouteRequest,
    RobotFailoverDeleteRouteRequest,
);

impl CheckedRobotFailover<'_, '_, RobotFailoverListRequest> {
    /// Decodes a bounded list of distinct failover routes.
    pub fn decode_response(self) -> Result<RobotFailoverList, RobotFailoverDecodeError> {
        self.inner
            .decode_owned_with_workspace(decode_robot_failover_list)
    }
}

impl CheckedRobotFailover<'_, '_, RobotFailoverGetRequest> {
    /// Decodes and binds one failover resource to the request route.
    pub fn decode_response(self) -> Result<RobotFailover, RobotFailoverDecodeError> {
        decode_bound(self.request.route(), self.inner)
    }
}

impl CheckedRobotFailover<'_, '_, RobotFailoverRerouteRequest> {
    /// Requires the provider to acknowledge the exact requested destination.
    pub fn decode_response(self) -> Result<RobotFailover, RobotFailoverDecodeError> {
        let result = decode_bound(self.request.route(), self.inner)?;
        let matches = result.with_active_server(|actual| {
            actual.is_some_and(|actual| {
                self.request
                    .active_server
                    .with_addr(|expected| actual == expected)
            })
        });
        if matches {
            Ok(result)
        } else {
            Err(RobotFailoverDecodeError::MutationOutcomeMismatch)
        }
    }
}

impl CheckedRobotFailover<'_, '_, RobotFailoverDeleteRouteRequest> {
    /// Requires the provider's JSON acknowledgement to contain no active route.
    pub fn decode_response(self) -> Result<RobotFailover, RobotFailoverDecodeError> {
        let result = decode_bound(self.request.route(), self.inner)?;
        if result.with_active_server(|actual| actual.is_none()) {
            Ok(result)
        } else {
            Err(RobotFailoverDecodeError::MutationOutcomeMismatch)
        }
    }
}

fn decode_bound(
    expected: &crate::robot::RobotIpAddress,
    checked: CheckedResponseGuard<'_>,
) -> Result<RobotFailover, RobotFailoverDecodeError> {
    checked.decode_owned_with_workspace(|response, workspace| {
        decode_robot_failover(response, expected, workspace)
    })
}

#[cfg(doctest)]
mod compile_fail {
    /// Different operation types cannot consume each other's checked response.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::{
    ///     CheckedRobotFailover, RobotFailoverGetRequest, RobotFailoverRerouteRequest,
    /// };
    /// fn consume(_: CheckedRobotFailover<'_, '_, RobotFailoverGetRequest>) {}
    /// fn wrong(response: CheckedRobotFailover<'_, '_, RobotFailoverRerouteRequest>) {
    ///     consume(response);
    /// }
    /// ```
    fn association() {}
}
