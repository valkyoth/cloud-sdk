use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PrepareOperation, PreparedRequest,
    ResponsePolicyError,
};
use cloud_sdk::transport::ResponseBuffer;

use super::decode::{
    RobotCancellationDecodeError, decode_robot_ip_cancellation, decode_robot_server_cancellation,
    decode_robot_subnet_cancellation,
};
use super::model::{RobotIpCancellation, RobotServerCancellation, RobotSubnetCancellation};
use super::request::*;

/// Prepared cancellation retaining the exact request instance used for policy.
///
/// Call [`Self::validate_response`] instead of validating through an untyped
/// [`PreparedRequest`] when decoding a cancellation acknowledgement.
pub struct PreparedCancellation<'storage, 'request, R> {
    request: &'request R,
    inner: PreparedRequest<'storage>,
}

impl<'storage, 'request, R> PreparedCancellation<'storage, 'request, R> {
    /// Borrows the provider-neutral request for inspection.
    ///
    /// Destructive authority must be built from
    /// [`super::CancellationPlanConfirmation`] so the exact request binding
    /// survives permit execution.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'storage> {
        self.inner
    }

    /// Applies this exact request's response policy and retains its association.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedCancellation<'buffer, 'request, R>, ResponsePolicyError> {
        self.inner
            .validate_response(response)
            .map(|inner| CheckedCancellation {
                request: self.request,
                inner,
            })
    }

    pub(super) fn into_plan_parts(self) -> (PreparedRequest<'storage>, &'request R) {
        (self.inner, self.request)
    }
}

impl<R> core::fmt::Debug for PreparedCancellation<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PreparedCancellation")
            .field("request", &"[bound]")
            .field("prepared", &self.inner)
            .finish()
    }
}

/// Checked response retaining the exact cancellation request that admitted it.
///
/// Operation types cannot be cross-wired:
///
/// ```compile_fail
/// use cloud_sdk_hetzner::robot::{
///     CheckedCancellation, RobotIpCancellationGetRequest,
///     RobotServerCancellationGetRequest,
/// };
///
/// fn wrong(
///     response: CheckedCancellation<'_, '_, RobotIpCancellationGetRequest>,
/// ) {
///     let _: cloud_sdk_hetzner::robot::RobotServerCancellation =
///         response.decode_response().unwrap();
/// }
/// ```
pub struct CheckedCancellation<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
}

impl<'buffer, 'request, R> CheckedCancellation<'buffer, 'request, R> {
    pub(super) const fn from_executed(
        request: &'request R,
        inner: CheckedResponseGuard<'buffer>,
    ) -> Self {
        Self { request, inner }
    }
}

impl<R> core::fmt::Debug for CheckedCancellation<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CheckedCancellation")
            .field("request", &"[bound]")
            .field("response", &"[checked]")
            .finish()
    }
}

macro_rules! prepare_bound {
    ($($type:ty),+ $(,)?) => {$ (
        impl $type {
            /// Prepares this request while retaining its response association.
            pub fn prepare_bound<'storage, 'request>(
                &'request self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedCancellation<'storage, 'request, Self>, RobotCancellationRequestError> {
                let inner = self.prepare(storage)?;
                Ok(PreparedCancellation { request: self, inner })
            }
        }
    )+ };
}

prepare_bound!(
    RobotServerCancellationGetRequest,
    RobotServerCancellationCreateRequest<'_>,
    RobotServerCancellationDeleteRequest,
    RobotIpCancellationGetRequest,
    RobotIpCancellationCreateRequest,
    RobotIpCancellationDeleteRequest,
    RobotSubnetCancellationGetRequest,
    RobotSubnetCancellationCreateRequest,
    RobotSubnetCancellationDeleteRequest,
);

macro_rules! get_decoder {
    ($request:ty, $output:ty, $decoder:ident, $field:ident) => {
        impl CheckedCancellation<'_, '_, $request> {
            /// Decodes, identity-checks, and clears this request's response.
            pub fn decode_response(self) -> Result<$output, RobotCancellationDecodeError> {
                self.inner
                    .decode_owned_with_workspace(|response, workspace| {
                        $decoder(response, &self.request.$field, workspace)
                    })
            }
        }
    };
}

get_decoder!(
    RobotServerCancellationGetRequest,
    RobotServerCancellation,
    decode_robot_server_cancellation,
    number
);
get_decoder!(
    RobotIpCancellationGetRequest,
    RobotIpCancellation,
    decode_robot_ip_cancellation,
    ip
);
get_decoder!(
    RobotSubnetCancellationGetRequest,
    RobotSubnetCancellation,
    decode_robot_subnet_cancellation,
    subnet
);

impl CheckedCancellation<'_, '_, RobotServerCancellationCreateRequest<'_>> {
    /// Decodes and verifies the complete server cancellation intent.
    pub fn decode_response(self) -> Result<RobotServerCancellation, RobotCancellationDecodeError> {
        let request = self.request;
        let result = self
            .inner
            .decode_owned_with_workspace(|response, workspace| {
                decode_robot_server_cancellation(response, &request.number, workspace)
            })?;
        validate_schedule(
            &request.schedule,
            result.is_cancelled(),
            result.cancellation_date(),
        )?;
        validate_reservation(
            request.reservation,
            result.reservation_possible(),
            result.is_reserved(),
        )?;
        validate_reason(request.reason, &result)?;
        Ok(result)
    }
}

impl CheckedCancellation<'_, '_, RobotIpCancellationCreateRequest> {
    /// Decodes and verifies the complete IP cancellation intent.
    pub fn decode_response(self) -> Result<RobotIpCancellation, RobotCancellationDecodeError> {
        let request = self.request;
        let result = self
            .inner
            .decode_owned_with_workspace(|response, workspace| {
                decode_robot_ip_cancellation(response, &request.ip, workspace)
            })?;
        validate_schedule(
            &request.schedule,
            result.is_cancelled(),
            result.cancellation_date(),
        )?;
        Ok(result)
    }
}

impl CheckedCancellation<'_, '_, RobotSubnetCancellationCreateRequest> {
    /// Decodes and verifies the complete subnet cancellation intent.
    pub fn decode_response(self) -> Result<RobotSubnetCancellation, RobotCancellationDecodeError> {
        let request = self.request;
        let result = self
            .inner
            .decode_owned_with_workspace(|response, workspace| {
                decode_robot_subnet_cancellation(response, &request.subnet, workspace)
            })?;
        validate_schedule(
            &request.schedule,
            result.is_cancelled(),
            result.cancellation_date(),
        )?;
        Ok(result)
    }
}

impl CheckedCancellation<'_, '_, RobotServerCancellationDeleteRequest> {
    /// Accepts and clears the exact empty server revocation acknowledgement.
    pub fn decode_response(self) -> Result<(), RobotCancellationDecodeError> {
        drop(self);
        Ok(())
    }
}

macro_rules! delete_decoder {
    ($request:ty, $output:ty, $decoder:ident, $field:ident) => {
        impl CheckedCancellation<'_, '_, $request> {
            /// Decodes and verifies an inactive revocation acknowledgement.
            pub fn decode_response(self) -> Result<$output, RobotCancellationDecodeError> {
                let request = self.request;
                let result = self
                    .inner
                    .decode_owned_with_workspace(|response, workspace| {
                        $decoder(response, &request.$field, workspace)
                    })?;
                if result.is_cancelled() {
                    return Err(RobotCancellationDecodeError::MutationOutcomeMismatch);
                }
                Ok(result)
            }
        }
    };
}

delete_decoder!(
    RobotIpCancellationDeleteRequest,
    RobotIpCancellation,
    decode_robot_ip_cancellation,
    ip
);
delete_decoder!(
    RobotSubnetCancellationDeleteRequest,
    RobotSubnetCancellation,
    decode_robot_subnet_cancellation,
    subnet
);

fn validate_schedule(
    schedule: &RobotCancellationSchedule,
    cancelled: bool,
    actual: Option<&super::RobotCancellationDate>,
) -> Result<(), RobotCancellationDecodeError> {
    let matches = match schedule {
        RobotCancellationSchedule::Immediate => cancelled && actual.is_some(),
        RobotCancellationSchedule::On(expected) => cancelled && actual == Some(expected),
    };
    if matches {
        Ok(())
    } else {
        Err(RobotCancellationDecodeError::MutationOutcomeMismatch)
    }
}

fn validate_reservation(
    requested: RobotLocationReservationIntent,
    reservation_possible: bool,
    reserved: bool,
) -> Result<(), RobotCancellationDecodeError> {
    let matches = match requested {
        RobotLocationReservationIntent::Omit => !reservation_possible && !reserved,
        RobotLocationReservationIntent::Reserve => reservation_possible && reserved,
        RobotLocationReservationIntent::DoNotReserve => !reserved,
    };
    if matches {
        Ok(())
    } else {
        Err(RobotCancellationDecodeError::MutationOutcomeMismatch)
    }
}

fn validate_reason(
    requested: Option<RobotCancellationReason<'_>>,
    result: &RobotServerCancellation,
) -> Result<(), RobotCancellationDecodeError> {
    let matches = match (requested, result.reason().selected()) {
        (None, Some(None)) => true,
        (Some(expected), Some(Some(actual))) => actual
            .try_with_secret(|value| value == expected.as_str())
            .unwrap_or(false),
        _ => false,
    };
    if matches {
        Ok(())
    } else {
        Err(RobotCancellationDecodeError::MutationOutcomeMismatch)
    }
}
