use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PrepareOperation, PreparedRequest,
    ResponsePolicyError,
};
use cloud_sdk::transport::ResponseBuffer;

use super::decode::{
    RobotSubnetDecodeError, decode_robot_subnet, decode_robot_subnet_list, decode_robot_subnet_mac,
};
use super::model::{RobotSubnet, RobotSubnetList, RobotSubnetMac};
use super::request::*;

/// Prepared Robot subnet request retaining its exact typed association.
pub struct PreparedRobotSubnet<'storage, 'request, R> {
    request: &'request R,
    inner: PreparedRequest<'storage>,
}

impl<'storage, 'request, R> PreparedRobotSubnet<'storage, 'request, R> {
    /// Borrows the provider-neutral prepared request for inspection.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'storage> {
        self.inner
    }

    /// Applies the exact prepared response policy and retains request provenance.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedRobotSubnet<'buffer, 'request, R>, ResponsePolicyError> {
        self.inner
            .validate_response(response)
            .map(|inner| CheckedRobotSubnet {
                request: self.request,
                inner,
            })
    }

    pub(super) fn into_plan_parts(self) -> (PreparedRequest<'storage>, &'request R) {
        (self.inner, self.request)
    }
}

impl<R> core::fmt::Debug for PreparedRobotSubnet<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PreparedRobotSubnet")
            .field("request", &"[bound]")
            .field("prepared", &self.inner)
            .finish()
    }
}

/// Checked Robot subnet response retaining the exact request that admitted it.
pub struct CheckedRobotSubnet<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
}

impl<'buffer, 'request, R> CheckedRobotSubnet<'buffer, 'request, R> {
    pub(super) const fn from_executed(
        request: &'request R,
        inner: CheckedResponseGuard<'buffer>,
    ) -> Self {
        Self { request, inner }
    }
}

impl<R> core::fmt::Debug for CheckedRobotSubnet<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CheckedRobotSubnet")
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
            ) -> Result<PreparedRobotSubnet<'storage, 'request, Self>, RobotSubnetRequestError> {
                let inner = self.prepare(storage)?;
                Ok(PreparedRobotSubnet { request: self, inner })
            }
        }
    )+ };
}

prepare_bound!(
    RobotSubnetListRequest,
    RobotSubnetGetRequest,
    RobotSubnetUpdateRequest,
    RobotSubnetMacGetRequest,
    RobotSubnetMacSetRequest,
    RobotSubnetMacDeleteRequest,
);

impl CheckedRobotSubnet<'_, '_, RobotSubnetListRequest> {
    /// Decodes and enforces the optional server-address filter.
    pub fn decode_response(self) -> Result<RobotSubnetList, RobotSubnetDecodeError> {
        let result = self
            .inner
            .decode_owned_with_workspace(decode_robot_subnet_list)?;
        if let Some(expected) = self.request.server_address.as_ref()
            && result.as_slice().iter().any(|entry| {
                entry.with_server_address(|actual| {
                    actual.is_none_or(|actual| expected.with_addr(|expected| actual != expected))
                })
            })
        {
            return Err(RobotSubnetDecodeError::ResponseIdentityMismatch);
        }
        Ok(result)
    }
}

impl CheckedRobotSubnet<'_, '_, RobotSubnetGetRequest> {
    /// Decodes and binds one detailed resource to the request address.
    pub fn decode_response(self) -> Result<RobotSubnet, RobotSubnetDecodeError> {
        self.inner
            .decode_owned_with_workspace(|response, workspace| {
                decode_robot_subnet(response, &self.request.address, workspace)
            })
    }
}

impl CheckedRobotSubnet<'_, '_, RobotSubnetUpdateRequest> {
    /// Decodes and verifies every explicitly requested traffic-policy field.
    pub fn decode_response(self) -> Result<RobotSubnet, RobotSubnetDecodeError> {
        let result = self
            .inner
            .decode_owned_with_workspace(|response, workspace| {
                decode_robot_subnet(response, &self.request.address, workspace)
            })?;
        let actual = result.traffic();
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
            Err(RobotSubnetDecodeError::MutationOutcomeMismatch)
        }
    }
}

impl CheckedRobotSubnet<'_, '_, RobotSubnetMacGetRequest> {
    /// Decodes the current MAC and its bounded selectable map.
    pub fn decode_response(self) -> Result<RobotSubnetMac, RobotSubnetDecodeError> {
        decode_mac_response(self.request.address(), self.inner)
    }
}

impl CheckedRobotSubnet<'_, '_, RobotSubnetMacSetRequest> {
    /// Decodes and verifies the explicitly selected MAC.
    pub fn decode_response(self) -> Result<RobotSubnetMac, RobotSubnetDecodeError> {
        let result = decode_mac_response(self.request.address(), self.inner)?;
        if result.mac() == self.request.mac() {
            Ok(result)
        } else {
            Err(RobotSubnetDecodeError::MutationOutcomeMismatch)
        }
    }
}

impl CheckedRobotSubnet<'_, '_, RobotSubnetMacDeleteRequest> {
    /// Decodes the restored default MAC and selectable map.
    pub fn decode_response(self) -> Result<RobotSubnetMac, RobotSubnetDecodeError> {
        decode_mac_response(self.request.address(), self.inner)
    }
}

fn decode_mac_response(
    expected: &crate::robot::RobotSubnetAddress,
    checked: CheckedResponseGuard<'_>,
) -> Result<RobotSubnetMac, RobotSubnetDecodeError> {
    checked.decode_owned_with_workspace(|response, workspace| {
        decode_robot_subnet_mac(response, expected, workspace)
    })
}

#[cfg(doctest)]
mod compile_fail {
    /// Different operation types cannot consume each other's checked response.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::{CheckedRobotSubnet, RobotSubnetGetRequest, RobotSubnetMacGetRequest};
    /// fn wrong(response: CheckedRobotSubnet<'_, '_, RobotSubnetMacGetRequest>) {
    ///     let _: cloud_sdk_hetzner::robot::RobotSubnet = response.decode_response().unwrap();
    /// }
    /// ```
    fn association() {}
}
