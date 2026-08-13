use super::{MAX_ROBOT_BOOT_AUTHORIZED_KEYS, RobotBootKey, RobotBootValue, RobotKeyboardLayout};
use crate::robot::RobotServerNumber;
use cloud_sdk::rate_limit::DelaySeconds;

/// Source-locked quota shared by all Robot boot operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RobotBootQuota {
    max_requests: u64,
    interval: DelaySeconds,
}

impl RobotBootQuota {
    /// Returns the maximum request count.
    #[must_use]
    pub const fn max_requests(self) -> u64 {
        self.max_requests
    }
    /// Returns the fixed quota interval.
    #[must_use]
    pub const fn interval(self) -> DelaySeconds {
        self.interval
    }
}

/// Five hundred requests per one-hour window.
pub const ROBOT_BOOT_QUOTA: RobotBootQuota = RobotBootQuota {
    max_requests: 500,
    interval: DelaySeconds::new(3_600),
};

/// Failure while validating or preparing a Robot boot operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotBootRequestError {
    /// Too many authorized SSH keys were supplied.
    TooManyAuthorizedKeys,
    /// An authorized SSH-key fingerprint was repeated.
    DuplicateAuthorizedKey,
    /// Temporary field metadata could not be allocated.
    Allocation,
    /// Caller-owned path storage was too small or encoding failed.
    Path,
    /// Robot form validation or encoding failed.
    Form(crate::robot::RobotFormError),
    /// Source-locked request headers were rejected.
    InvalidHeaders(cloud_sdk::transport::HeaderError),
    /// The official Robot endpoint policy was invalid.
    InvalidEndpoint(crate::endpoint::OfficialEndpointError),
    /// A source-locked operation identifier was invalid.
    InvalidOperationId(cloud_sdk::operation::OperationIdError),
    /// Operation safety metadata was internally inconsistent.
    InvalidMetadata(cloud_sdk::operation::OperationMetadataError),
    /// The success-response policy was internally inconsistent.
    InvalidResponsePolicy(cloud_sdk::operation::ResponsePolicyValidationError),
    /// The raw response-wire policy was internally inconsistent.
    InvalidRawPolicy(cloud_sdk::transport::RawResponsePolicyError),
    /// Cross-policy prepared-request validation failed.
    InvalidPreparedPolicy(cloud_sdk::operation::PreparedRequestPolicyError),
}

impl_static_error!(RobotBootRequestError,
    Self::TooManyAuthorizedKeys => "Robot boot activation has too many authorized keys",
    Self::DuplicateAuthorizedKey => "Robot boot activation repeats an authorized key",
    Self::Allocation => "Robot boot form metadata allocation failed",
    Self::Path => "Robot boot path preparation failed",
    Self::Form(_) => "Robot boot form preparation failed",
    Self::InvalidHeaders(_) => "Robot boot headers are invalid",
    Self::InvalidEndpoint(_) => "official Robot endpoint is invalid",
    Self::InvalidOperationId(_) => "Robot boot operation identifier is invalid",
    Self::InvalidMetadata(_) => "Robot boot metadata is invalid",
    Self::InvalidResponsePolicy(_) => "Robot boot response policy is invalid",
    Self::InvalidRawPolicy(_) => "Robot boot raw response policy is invalid",
    Self::InvalidPreparedPolicy(_) => "Robot boot prepared policy is invalid",
);

macro_rules! number_request {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        pub struct $name {
            pub(super) number: RobotServerNumber,
        }

        impl $name {
            /// Creates a request for one canonical server number.
            #[must_use]
            pub const fn new(number: RobotServerNumber) -> Self {
                Self { number }
            }

            /// Returns the canonical requested server number.
            #[must_use]
            pub const fn number(&self) -> &RobotServerNumber {
                &self.number
            }

            /// Returns the source-locked request quota.
            #[must_use]
            pub const fn quota(&self) -> RobotBootQuota {
                ROBOT_BOOT_QUOTA
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(concat!(stringify!($name), "([redacted])"))
            }
        }
    };
}

number_request!(RobotBootGetRequest, "Gets all boot configuration families.");
number_request!(RobotRescueGetRequest, "Gets Rescue boot options.");
number_request!(RobotRescueDeactivateRequest, "Deactivates Rescue boot.");
number_request!(RobotRescueLastRequest, "Gets the last Rescue activation.");
number_request!(RobotLinuxGetRequest, "Gets Linux installation options.");
number_request!(
    RobotLinuxDeactivateRequest,
    "Deactivates Linux installation."
);
number_request!(RobotLinuxLastRequest, "Gets the last Linux activation.");
number_request!(RobotVncGetRequest, "Gets VNC installation options.");
number_request!(RobotVncDeactivateRequest, "Deactivates VNC installation.");
number_request!(RobotWindowsGetRequest, "Gets Windows installation options.");
number_request!(
    RobotWindowsDeactivateRequest,
    "Deactivates Windows installation."
);

/// Activates Rescue boot with explicit OS, optional keys, and keyboard layout.
pub struct RobotRescueActivateRequest<'a> {
    pub(super) number: RobotServerNumber,
    pub(super) os: RobotBootValue<'a>,
    pub(super) keys: &'a [RobotBootKey<'a>],
    pub(super) keyboard: Option<RobotKeyboardLayout<'a>>,
}

impl<'a> RobotRescueActivateRequest<'a> {
    /// Creates a bounded Rescue activation.
    pub fn new(
        number: RobotServerNumber,
        os: RobotBootValue<'a>,
        keys: &'a [RobotBootKey<'a>],
        keyboard: Option<RobotKeyboardLayout<'a>>,
    ) -> Result<Self, RobotBootRequestError> {
        validate_keys(keys)?;
        Ok(Self {
            number,
            os,
            keys,
            keyboard,
        })
    }

    /// Returns the canonical requested server number.
    #[must_use]
    pub const fn number(&self) -> &RobotServerNumber {
        &self.number
    }

    /// Returns the source-locked request quota.
    #[must_use]
    pub const fn quota(&self) -> RobotBootQuota {
        ROBOT_BOOT_QUOTA
    }
}

/// Activates a Linux installation with explicit distribution and language.
pub struct RobotLinuxActivateRequest<'a> {
    pub(super) number: RobotServerNumber,
    pub(super) distribution: RobotBootValue<'a>,
    pub(super) language: RobotBootValue<'a>,
    pub(super) keys: &'a [RobotBootKey<'a>],
}

impl<'a> RobotLinuxActivateRequest<'a> {
    /// Creates a bounded Linux activation.
    pub fn new(
        number: RobotServerNumber,
        distribution: RobotBootValue<'a>,
        language: RobotBootValue<'a>,
        keys: &'a [RobotBootKey<'a>],
    ) -> Result<Self, RobotBootRequestError> {
        validate_keys(keys)?;
        Ok(Self {
            number,
            distribution,
            language,
            keys,
        })
    }

    /// Returns the canonical requested server number.
    #[must_use]
    pub const fn number(&self) -> &RobotServerNumber {
        &self.number
    }

    /// Returns the source-locked request quota.
    #[must_use]
    pub const fn quota(&self) -> RobotBootQuota {
        ROBOT_BOOT_QUOTA
    }
}

/// Activates a VNC installation with explicit distribution and language.
pub struct RobotVncActivateRequest<'a> {
    pub(super) number: RobotServerNumber,
    pub(super) distribution: RobotBootValue<'a>,
    pub(super) language: RobotBootValue<'a>,
}

impl<'a> RobotVncActivateRequest<'a> {
    /// Creates a bounded VNC activation.
    #[must_use]
    pub const fn new(
        number: RobotServerNumber,
        distribution: RobotBootValue<'a>,
        language: RobotBootValue<'a>,
    ) -> Self {
        Self {
            number,
            distribution,
            language,
        }
    }

    /// Returns the canonical requested server number.
    #[must_use]
    pub const fn number(&self) -> &RobotServerNumber {
        &self.number
    }

    /// Returns the source-locked request quota.
    #[must_use]
    pub const fn quota(&self) -> RobotBootQuota {
        ROBOT_BOOT_QUOTA
    }
}

/// Activates a destructive Windows installation.
pub struct RobotWindowsActivateRequest<'a> {
    pub(super) number: RobotServerNumber,
    pub(super) operating_system: RobotBootValue<'a>,
    pub(super) language: RobotBootValue<'a>,
}

impl<'a> RobotWindowsActivateRequest<'a> {
    /// Creates an explicit Windows installation activation.
    #[must_use]
    pub const fn new(
        number: RobotServerNumber,
        operating_system: RobotBootValue<'a>,
        language: RobotBootValue<'a>,
    ) -> Self {
        Self {
            number,
            operating_system,
            language,
        }
    }

    /// Returns the canonical requested server number.
    #[must_use]
    pub const fn number(&self) -> &RobotServerNumber {
        &self.number
    }

    /// Returns the source-locked request quota.
    #[must_use]
    pub const fn quota(&self) -> RobotBootQuota {
        ROBOT_BOOT_QUOTA
    }
}

fn validate_keys(keys: &[RobotBootKey<'_>]) -> Result<(), RobotBootRequestError> {
    if keys.len() > MAX_ROBOT_BOOT_AUTHORIZED_KEYS {
        return Err(RobotBootRequestError::TooManyAuthorizedKeys);
    }
    for (index, key) in keys.iter().enumerate() {
        let Some(previous) = keys.get(..index) else {
            unreachable!("enumerated Robot boot key prefix exceeded collection")
        };
        if previous.contains(key) {
            return Err(RobotBootRequestError::DuplicateAuthorizedKey);
        }
    }
    Ok(())
}

macro_rules! redacted_debug {
    ($($name:ident),+ $(,)?) => {$ (
        impl core::fmt::Debug for $name<'_> {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(concat!(stringify!($name), "([redacted])"))
            }
        }
    )+ };
}

redacted_debug!(
    RobotRescueActivateRequest,
    RobotLinuxActivateRequest,
    RobotVncActivateRequest,
    RobotWindowsActivateRequest,
);
