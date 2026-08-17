use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PrepareOperation, PreparedRequest,
    ResponsePolicyError,
};
use cloud_sdk::transport::ResponseBuffer;

use super::decode::{RobotVSwitchDecodeError, decode_robot_vswitch, decode_robot_vswitch_list};
use super::model::{RobotVSwitch, RobotVSwitchList};
use super::request::*;

/// Prepared Robot vSwitch request retaining its exact typed association.
pub struct PreparedRobotVSwitch<'storage, 'request, R> {
    request: &'request R,
    inner: PreparedRequest<'storage>,
}

impl<'storage, 'request, R> PreparedRobotVSwitch<'storage, 'request, R> {
    /// Borrows the provider-neutral prepared request for inspection.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'storage> {
        self.inner
    }

    /// Applies the exact response policy and retains request provenance.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedRobotVSwitch<'buffer, 'request, R>, ResponsePolicyError> {
        self.inner
            .validate_response(response)
            .map(|inner| CheckedRobotVSwitch {
                request: self.request,
                inner,
            })
    }

    pub(super) fn into_plan_parts(self) -> (PreparedRequest<'storage>, &'request R) {
        (self.inner, self.request)
    }
}

impl<R> core::fmt::Debug for PreparedRobotVSwitch<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PreparedRobotVSwitch")
            .field("request", &"[bound]")
            .field("prepared", &self.inner)
            .finish()
    }
}

/// Checked Robot vSwitch response retaining its admitting request.
pub struct CheckedRobotVSwitch<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
}

impl<'buffer, 'request, R> CheckedRobotVSwitch<'buffer, 'request, R> {
    pub(crate) const fn from_executed(
        request: &'request R,
        inner: CheckedResponseGuard<'buffer>,
    ) -> Self {
        Self { request, inner }
    }
}

impl<R> core::fmt::Debug for CheckedRobotVSwitch<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CheckedRobotVSwitch")
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
            ) -> Result<PreparedRobotVSwitch<'storage, 'request, Self>, RobotVSwitchRequestError> {
                let inner = self.prepare(storage)?;
                Ok(PreparedRobotVSwitch { request: self, inner })
            }
        }
    )+ };
}

prepare_bound!(
    RobotVSwitchListRequest,
    RobotVSwitchCreateRequest,
    RobotVSwitchGetRequest,
    RobotVSwitchUpdateRequest,
    RobotVSwitchCancelRequest,
    RobotVSwitchAddServersRequest<'_>,
    RobotVSwitchRemoveServersRequest<'_>,
);

impl CheckedRobotVSwitch<'_, '_, RobotVSwitchListRequest> {
    /// Decodes a bounded inventory with unique vSwitch IDs.
    pub fn decode_response(self) -> Result<RobotVSwitchList, RobotVSwitchDecodeError> {
        self.inner
            .decode_owned_with_workspace(decode_robot_vswitch_list)
    }
}

impl CheckedRobotVSwitch<'_, '_, RobotVSwitchGetRequest> {
    /// Decodes one vSwitch and binds it to the requested identity.
    pub fn decode_response(self) -> Result<RobotVSwitch, RobotVSwitchDecodeError> {
        let result = decode_one(self.inner)?;
        require_id(&result, self.request.id)?;
        Ok(result)
    }
}

impl CheckedRobotVSwitch<'_, '_, RobotVSwitchCreateRequest> {
    /// Requires the created resource to match both requested configuration fields.
    pub fn decode_response(self) -> Result<RobotVSwitch, RobotVSwitchDecodeError> {
        let result = decode_one(self.inner)?;
        if result.name.matches(&self.request.name)
            && result.vlan == self.request.vlan
            && !result.cancelled
            && result.servers.is_empty()
            && result.subnets.is_empty()
            && result.cloud_networks.is_empty()
        {
            Ok(result)
        } else {
            Err(RobotVSwitchDecodeError::MutationOutcomeMismatch)
        }
    }
}

macro_rules! empty_acknowledgement {
    ($($type:ty),+ $(,)?) => {$ (
        impl CheckedRobotVSwitch<'_, '_, $type> {
            /// Accepts the exact source-locked empty acknowledgement.
            ///
            /// Callers must reconcile the resulting provider state with a
            /// subsequent `GET /vswitch/{id}` before dependent automation.
            pub fn decode_response(self) -> Result<(), RobotVSwitchDecodeError> {
                drop(self);
                Ok(())
            }
        }
    )+ };
}

empty_acknowledgement!(
    RobotVSwitchUpdateRequest,
    RobotVSwitchCancelRequest,
    RobotVSwitchAddServersRequest<'_>,
    RobotVSwitchRemoveServersRequest<'_>,
);

fn decode_one(checked: CheckedResponseGuard<'_>) -> Result<RobotVSwitch, RobotVSwitchDecodeError> {
    checked.decode_owned_with_workspace(decode_robot_vswitch)
}

fn require_id(
    result: &RobotVSwitch,
    expected: super::RobotVSwitchId,
) -> Result<(), RobotVSwitchDecodeError> {
    if result.id == expected {
        Ok(())
    } else {
        Err(RobotVSwitchDecodeError::ResponseIdentityMismatch)
    }
}

#[cfg(doctest)]
mod compile_fail {
    /// Different operation types cannot consume each other's checked response.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::{
    ///     CheckedRobotVSwitch, RobotVSwitchGetRequest, RobotVSwitchUpdateRequest,
    /// };
    /// fn consume(_: CheckedRobotVSwitch<'_, '_, RobotVSwitchGetRequest>) {}
    /// fn wrong(response: CheckedRobotVSwitch<'_, '_, RobotVSwitchUpdateRequest>) {
    ///     consume(response);
    /// }
    /// ```
    fn association() {}
}
