use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PrepareOperation, PreparedRequest,
    ResponsePolicyError,
};
use cloud_sdk::transport::ResponseBuffer;

use super::decode::state::RobotBootEntryShape;
use super::decode::{RobotBootDecodeError, decode_robot_boot, decode_robot_boot_entry};
use super::model::{RobotBoot, RobotBootEntry, RobotBootFamily};
use super::request::*;

/// Prepared Robot boot request retaining its exact typed association.
pub struct PreparedRobotBoot<'storage, 'request, R> {
    pub(super) request: &'request R,
    pub(super) inner: PreparedRequest<'storage>,
}

impl<'storage, 'request, R> PreparedRobotBoot<'storage, 'request, R> {
    /// Borrows the provider-neutral prepared request for inspection.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'storage> {
        self.inner
    }

    /// Applies the exact response policy and retains request provenance.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedRobotBoot<'buffer, 'request, R>, ResponsePolicyError> {
        self.inner
            .validate_response(response)
            .map(|inner| CheckedRobotBoot {
                request: self.request,
                inner,
            })
    }
}

impl<R> core::fmt::Debug for PreparedRobotBoot<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PreparedRobotBoot")
            .field("request", &"[bound]")
            .field("prepared", &self.inner)
            .finish()
    }
}

/// Checked Robot boot response retaining the request that admitted it.
pub struct CheckedRobotBoot<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
}

impl<R> core::fmt::Debug for CheckedRobotBoot<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CheckedRobotBoot")
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
            ) -> Result<PreparedRobotBoot<'storage, 'request, Self>, RobotBootRequestError> {
                let inner = self.prepare(storage)?;
                Ok(PreparedRobotBoot { request: self, inner })
            }
        }
    )+ };
}

prepare_bound!(
    RobotBootGetRequest,
    RobotRescueGetRequest,
    RobotRescueActivateRequest<'_>,
    RobotRescueDeactivateRequest,
    RobotRescueLastRequest,
    RobotLinuxGetRequest,
    RobotLinuxActivateRequest<'_>,
    RobotLinuxDeactivateRequest,
    RobotLinuxLastRequest,
    RobotVncGetRequest,
    RobotVncActivateRequest<'_>,
    RobotVncDeactivateRequest,
    RobotWindowsGetRequest,
    RobotWindowsActivateRequest<'_>,
    RobotWindowsDeactivateRequest,
);

impl CheckedRobotBoot<'_, '_, RobotBootGetRequest> {
    /// Decodes the complete four-family overview.
    pub fn decode_response(self) -> Result<RobotBoot, RobotBootDecodeError> {
        self.inner
            .decode_owned_with_workspace(|response, workspace| {
                decode_robot_boot(response, self.request.number(), workspace)
            })
    }
}

macro_rules! decode_read {
    ($type:ty, $family:expr, $shape:expr) => {
        impl CheckedRobotBoot<'_, '_, $type> {
            /// Decodes one identity-bound family response.
            pub fn decode_response(self) -> Result<RobotBootEntry, RobotBootDecodeError> {
                decode_entry(self.request.number(), $family, $shape, self.inner)
            }
        }
    };
}

decode_read!(
    RobotRescueGetRequest,
    RobotBootFamily::Rescue,
    RobotBootEntryShape::Current
);
decode_read!(
    RobotRescueLastRequest,
    RobotBootFamily::Rescue,
    RobotBootEntryShape::Last
);
decode_read!(
    RobotLinuxGetRequest,
    RobotBootFamily::Linux,
    RobotBootEntryShape::Current
);
decode_read!(
    RobotLinuxLastRequest,
    RobotBootFamily::Linux,
    RobotBootEntryShape::Last
);
decode_read!(
    RobotVncGetRequest,
    RobotBootFamily::Vnc,
    RobotBootEntryShape::Current
);
decode_read!(
    RobotWindowsGetRequest,
    RobotBootFamily::Windows,
    RobotBootEntryShape::Current
);

macro_rules! decode_deactivate {
    ($type:ty, $family:expr) => {
        impl CheckedRobotBoot<'_, '_, $type> {
            /// Decodes an exact inactive acknowledgement.
            pub fn decode_response(self) -> Result<RobotBootEntry, RobotBootDecodeError> {
                let entry = decode_entry(
                    self.request.number(),
                    $family,
                    RobotBootEntryShape::Deactivation,
                    self.inner,
                )?;
                require_inactive(entry)
            }
        }
    };
}

decode_deactivate!(RobotRescueDeactivateRequest, RobotBootFamily::Rescue);
decode_deactivate!(RobotLinuxDeactivateRequest, RobotBootFamily::Linux);
decode_deactivate!(RobotVncDeactivateRequest, RobotBootFamily::Vnc);
decode_deactivate!(RobotWindowsDeactivateRequest, RobotBootFamily::Windows);

impl CheckedRobotBoot<'_, '_, RobotRescueActivateRequest<'_>> {
    /// Decodes and matches an active Rescue acknowledgement.
    pub fn decode_response(self) -> Result<RobotBootEntry, RobotBootDecodeError> {
        let entry = decode_entry(
            self.request.number(),
            RobotBootFamily::Rescue,
            RobotBootEntryShape::Activation,
            self.inner,
        )?;
        require_active_choice(entry, self.request.os.as_str(), None)
    }
}

impl CheckedRobotBoot<'_, '_, RobotLinuxActivateRequest<'_>> {
    /// Decodes and matches an active Linux acknowledgement.
    pub fn decode_response(self) -> Result<RobotBootEntry, RobotBootDecodeError> {
        let entry = decode_entry(
            self.request.number(),
            RobotBootFamily::Linux,
            RobotBootEntryShape::Activation,
            self.inner,
        )?;
        require_active_choice(
            entry,
            self.request.distribution.as_str(),
            Some(self.request.language.as_str()),
        )
    }
}

impl CheckedRobotBoot<'_, '_, RobotVncActivateRequest<'_>> {
    /// Decodes and matches an active VNC acknowledgement.
    pub fn decode_response(self) -> Result<RobotBootEntry, RobotBootDecodeError> {
        let entry = decode_entry(
            self.request.number(),
            RobotBootFamily::Vnc,
            RobotBootEntryShape::Activation,
            self.inner,
        )?;
        require_active_choice(
            entry,
            self.request.distribution.as_str(),
            Some(self.request.language.as_str()),
        )
    }
}

impl CheckedRobotBoot<'_, '_, RobotWindowsActivateRequest<'_>> {
    /// Decodes and matches a destructive Windows acknowledgement.
    pub fn decode_response(self) -> Result<RobotBootEntry, RobotBootDecodeError> {
        let entry = decode_entry(
            self.request.number(),
            RobotBootFamily::Windows,
            RobotBootEntryShape::Activation,
            self.inner,
        )?;
        require_active_choice(
            entry,
            self.request.operating_system.as_str(),
            Some(self.request.language.as_str()),
        )
    }
}

fn decode_entry(
    expected: &crate::robot::RobotServerNumber,
    family: RobotBootFamily,
    shape: RobotBootEntryShape,
    checked: CheckedResponseGuard<'_>,
) -> Result<RobotBootEntry, RobotBootDecodeError> {
    checked.decode_owned_with_workspace(|response, workspace| {
        decode_robot_boot_entry(response, expected, family, shape, workspace)
    })
}

fn require_inactive(entry: RobotBootEntry) -> Result<RobotBootEntry, RobotBootDecodeError> {
    if entry.is_active() || entry.password().is_some() || entry.primary_choice().is_selected() {
        Err(RobotBootDecodeError::MutationOutcomeMismatch)
    } else {
        Ok(entry)
    }
}

fn require_active_choice(
    entry: RobotBootEntry,
    primary: &str,
    language: Option<&str>,
) -> Result<RobotBootEntry, RobotBootDecodeError> {
    let primary_matches = entry.primary_choice().get(0).is_some_and(|value| {
        value
            .try_with_secret(|actual| actual == primary)
            .unwrap_or(false)
    });
    let language_matches = match language {
        None => entry.languages().is_none(),
        Some(expected) => entry
            .languages()
            .and_then(|values| values.get(0))
            .is_some_and(|value| {
                value
                    .try_with_secret(|actual| actual == expected)
                    .unwrap_or(false)
            }),
    };
    if entry.is_active()
        && entry.password().is_some()
        && entry.primary_choice().is_selected()
        && primary_matches
        && language_matches
    {
        Ok(entry)
    } else {
        Err(RobotBootDecodeError::MutationOutcomeMismatch)
    }
}

#[cfg(doctest)]
mod compile_fail {
    /// Different boot operation types cannot consume each other's response.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::{
    ///     CheckedRobotBoot, RobotLinuxGetRequest, RobotWindowsGetRequest,
    /// };
    /// fn consume(_: CheckedRobotBoot<'_, '_, RobotLinuxGetRequest>) {}
    /// fn wrong(response: CheckedRobotBoot<'_, '_, RobotWindowsGetRequest>) {
    ///     consume(response);
    /// }
    /// ```
    fn typed_association() {}
}
