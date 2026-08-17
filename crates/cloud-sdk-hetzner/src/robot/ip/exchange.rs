use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PrepareOperation, PreparedRequest,
    ResponsePolicyError,
};
use cloud_sdk::transport::ResponseBuffer;

use super::decode::{
    RobotIpDecodeError, decode_robot_ip, decode_robot_ip_list, decode_robot_ip_mac,
};
use super::model::{RobotIp, RobotIpList, RobotIpMac};
use super::request::*;

/// Prepared Robot IP request retaining its exact typed association.
pub struct PreparedRobotIp<'storage, 'request, R> {
    request: &'request R,
    inner: PreparedRequest<'storage>,
}

impl<'storage, 'request, R> PreparedRobotIp<'storage, 'request, R> {
    /// Borrows the provider-neutral prepared request for inspection.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'storage> {
        self.inner
    }

    /// Applies the exact prepared response policy and retains request provenance.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedRobotIp<'buffer, 'request, R>, ResponsePolicyError> {
        self.inner
            .validate_response(response)
            .map(|inner| CheckedRobotIp {
                request: self.request,
                inner,
            })
    }

    pub(super) fn into_plan_parts(self) -> (PreparedRequest<'storage>, &'request R) {
        (self.inner, self.request)
    }
}

impl<R> core::fmt::Debug for PreparedRobotIp<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PreparedRobotIp")
            .field("request", &"[bound]")
            .field("prepared", &self.inner)
            .finish()
    }
}

/// Checked Robot IP response retaining the exact request that admitted it.
pub struct CheckedRobotIp<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
}

impl<'buffer, 'request, R> CheckedRobotIp<'buffer, 'request, R> {
    pub(crate) const fn from_executed(
        request: &'request R,
        inner: CheckedResponseGuard<'buffer>,
    ) -> Self {
        Self { request, inner }
    }
}

impl<R> core::fmt::Debug for CheckedRobotIp<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CheckedRobotIp")
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
            ) -> Result<PreparedRobotIp<'storage, 'request, Self>, RobotIpRequestError> {
                let inner = self.prepare(storage)?;
                Ok(PreparedRobotIp { request: self, inner })
            }
        }
    )+ };
}

prepare_bound!(
    RobotIpListRequest,
    RobotIpGetRequest,
    RobotIpUpdateRequest,
    RobotIpMacGetRequest,
    RobotIpMacSetRequest,
    RobotIpMacDeleteRequest,
);

impl CheckedRobotIp<'_, '_, RobotIpListRequest> {
    /// Decodes and enforces the optional server-address filter.
    pub fn decode_response(self) -> Result<RobotIpList, RobotIpDecodeError> {
        let result = self
            .inner
            .decode_owned_with_workspace(decode_robot_ip_list)?;
        if let Some(expected) = self.request.server_address.as_ref()
            && result.as_slice().iter().any(|entry| {
                !entry.with_server_address(|actual| expected.with_addr(|e| actual == e))
            })
        {
            return Err(RobotIpDecodeError::ResponseIdentityMismatch);
        }
        Ok(result)
    }
}

impl CheckedRobotIp<'_, '_, RobotIpGetRequest> {
    /// Decodes and binds one detailed resource to the request address.
    pub fn decode_response(self) -> Result<RobotIp, RobotIpDecodeError> {
        self.inner
            .decode_owned_with_workspace(|response, workspace| {
                decode_robot_ip(response, &self.request.address, workspace)
            })
    }
}

impl CheckedRobotIp<'_, '_, RobotIpUpdateRequest> {
    /// Decodes and verifies every explicitly requested traffic-policy field.
    pub fn decode_response(self) -> Result<RobotIp, RobotIpDecodeError> {
        let result = self
            .inner
            .decode_owned_with_workspace(|response, workspace| {
                decode_robot_ip(response, &self.request.address, workspace)
            })?;
        let actual = result.summary().traffic();
        let expected = self.request.update;
        let matches = expected
            .warnings
            .is_none_or(|value| actual.enabled() == value)
            && expected
                .hourly
                .is_none_or(|value| actual.hourly_megabytes() == value)
            && expected
                .daily
                .is_none_or(|value| actual.daily_megabytes() == value)
            && expected
                .monthly
                .is_none_or(|value| actual.monthly_gigabytes() == value);
        if matches {
            Ok(result)
        } else {
            Err(RobotIpDecodeError::MutationOutcomeMismatch)
        }
    }
}

impl CheckedRobotIp<'_, '_, RobotIpMacGetRequest> {
    /// Decodes an existing separate MAC and rejects `null` success.
    pub fn decode_response(self) -> Result<RobotIpMac, RobotIpDecodeError> {
        decode_mac_response(self.request.address(), self.inner, true)
    }
}

impl CheckedRobotIp<'_, '_, RobotIpMacSetRequest> {
    /// Decodes a generated separate MAC and rejects `null` success.
    pub fn decode_response(self) -> Result<RobotIpMac, RobotIpDecodeError> {
        decode_mac_response(self.request.address(), self.inner, true)
    }
}

impl CheckedRobotIp<'_, '_, RobotIpMacDeleteRequest> {
    /// Decodes a deletion acknowledgement and requires an exact `null` MAC.
    pub fn decode_response(self) -> Result<RobotIpMac, RobotIpDecodeError> {
        decode_mac_response(self.request.address(), self.inner, false)
    }
}

fn decode_mac_response(
    expected: &crate::robot::RobotIpAddress,
    checked: CheckedResponseGuard<'_>,
    must_exist: bool,
) -> Result<RobotIpMac, RobotIpDecodeError> {
    let result = checked.decode_owned_with_workspace(|response, workspace| {
        decode_robot_ip_mac(response, expected, workspace)
    })?;
    if result.mac().is_some() == must_exist {
        Ok(result)
    } else {
        Err(RobotIpDecodeError::MutationOutcomeMismatch)
    }
}

#[cfg(doctest)]
mod compile_fail {
    /// Different operation types cannot consume each other's checked response.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::{CheckedRobotIp, RobotIpGetRequest, RobotIpMacGetRequest};
    /// fn wrong(response: CheckedRobotIp<'_, '_, RobotIpMacGetRequest>) {
    ///     let _: cloud_sdk_hetzner::robot::RobotIp = response.decode_response().unwrap();
    /// }
    /// ```
    fn association() {}
}
