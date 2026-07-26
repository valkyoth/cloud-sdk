//! Extensible provider and service identities.

use core::fmt;

/// Maximum bytes in a provider identifier.
pub const MAX_PROVIDER_ID_BYTES: usize = 63;

/// Maximum bytes in a service identifier.
pub const MAX_SERVICE_ID_BYTES: usize = 63;

/// Invalid provider or service identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityError {
    /// The identifier is empty.
    Empty,
    /// The identifier exceeds its domain's byte limit.
    TooLong,
    /// The identifier contains a byte outside lowercase ASCII, digits, or `-`.
    InvalidByte,
    /// The identifier begins or ends with `-`.
    InvalidBoundary,
    /// The identifier contains consecutive `-` separators.
    RepeatedSeparator,
}

impl fmt::Display for IdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "provider or service identifier is empty",
            Self::TooLong => "provider or service identifier is too long",
            Self::InvalidByte => "provider or service identifier contains an invalid byte",
            Self::InvalidBoundary => "provider or service identifier has an invalid boundary",
            Self::RepeatedSeparator => {
                "provider or service identifier contains repeated separators"
            }
        })
    }
}

impl core::error::Error for IdentityError {}

/// Bounded canonical cloud-provider identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProviderId(&'static str);

impl ProviderId {
    /// Validates a static provider identifier.
    ///
    /// Identifiers use lowercase ASCII letters, digits, and single internal
    /// hyphens. This keeps equality independent of locale, Unicode
    /// normalization, and case folding.
    pub const fn new(value: &'static str) -> Result<Self, IdentityError> {
        match validate(value, MAX_PROVIDER_ID_BYTES) {
            Ok(()) => Ok(Self(value)),
            Err(error) => Err(error),
        }
    }

    /// Returns the canonical provider identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// Bounded canonical service identifier within a provider namespace.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ServiceId(&'static str);

impl ServiceId {
    /// Validates a static service identifier.
    ///
    /// Identifiers use lowercase ASCII letters, digits, and single internal
    /// hyphens. Service identity is meaningful only with its provider ID.
    pub const fn new(value: &'static str) -> Result<Self, IdentityError> {
        match validate(value, MAX_SERVICE_ID_BYTES) {
            Ok(()) => Ok(Self(value)),
            Err(error) => Err(error),
        }
    }

    /// Returns the canonical service identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// Provider-owned marker for an extensible provider namespace.
///
/// Provider crates implement this trait on their own zero-sized marker. The
/// neutral crate has no provider registry to edit.
///
/// ```
/// use cloud_sdk::{ProviderId, ProviderMarker, provider_id};
///
/// enum ExampleProvider {}
///
/// impl ProviderMarker for ExampleProvider {
///     const ID: ProviderId = provider_id!("example-cloud");
/// }
///
/// assert_eq!(ExampleProvider::ID.as_str(), "example-cloud");
/// ```
///
/// The private ID representation prevents bypassing validation.
///
/// ```compile_fail
/// use cloud_sdk::ProviderId;
///
/// const FORGED: ProviderId = ProviderId("UPPERCASE");
/// ```
pub trait ProviderMarker {
    /// Canonical provider identifier.
    const ID: ProviderId;
}

/// Provider-owned marker for one service namespace.
///
/// The associated provider makes service ownership explicit without a central
/// provider/service enum.
///
/// ```compile_fail
/// use cloud_sdk::{ServiceId, ServiceMarker, service_id};
///
/// enum MissingProvider {}
///
/// impl ServiceMarker for MissingProvider {
///     const ID: ServiceId = service_id!("compute");
/// }
/// ```
pub trait ServiceMarker {
    /// Provider that owns this service.
    type Provider: ProviderMarker;

    /// Canonical service identifier inside the provider namespace.
    const ID: ServiceId;
}

#[doc(hidden)]
pub const fn validate(value: &str, max_bytes: usize) -> Result<(), IdentityError> {
    if value.is_empty() {
        return Err(IdentityError::Empty);
    }
    if value.len() > max_bytes {
        return Err(IdentityError::TooLong);
    }
    let mut remaining = value.as_bytes();
    let mut first = true;
    let mut previous_separator = false;
    while let Some((byte, tail)) = remaining.split_first() {
        let separator = *byte == b'-';
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && !separator {
            return Err(IdentityError::InvalidByte);
        }
        if separator && first {
            return Err(IdentityError::InvalidBoundary);
        }
        if separator && previous_separator {
            return Err(IdentityError::RepeatedSeparator);
        }
        previous_separator = separator;
        first = false;
        remaining = tail;
    }
    if previous_separator {
        return Err(IdentityError::InvalidBoundary);
    }
    Ok(())
}

/// Creates a compile-time validated [`ProviderId`].
#[macro_export]
macro_rules! provider_id {
    ($value:literal) => {{
        const RESULT: Result<$crate::ProviderId, $crate::IdentityError> =
            $crate::ProviderId::new($value);
        match RESULT {
            Ok(value) => value,
            Err(_) => panic!("invalid provider identifier literal"),
        }
    }};
}

/// Creates a compile-time validated [`ServiceId`].
#[macro_export]
macro_rules! service_id {
    ($value:literal) => {{
        const RESULT: Result<$crate::ServiceId, $crate::IdentityError> =
            $crate::ServiceId::new($value);
        match RESULT {
            Ok(value) => value,
            Err(_) => panic!("invalid service identifier literal"),
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::{
        IdentityError, MAX_PROVIDER_ID_BYTES, MAX_SERVICE_ID_BYTES, ProviderId, ProviderMarker,
        ServiceId, ServiceMarker,
    };

    enum Example {}

    impl ProviderMarker for Example {
        const ID: ProviderId = provider_id!("example");
    }

    enum Compute {}

    impl ServiceMarker for Compute {
        type Provider = Example;
        const ID: ServiceId = service_id!("compute-v2");
    }

    #[test]
    fn markers_define_external_namespaces_without_core_registration() {
        assert_eq!(Example::ID.as_str(), "example");
        assert_eq!(Compute::ID.as_str(), "compute-v2");
        assert_eq!(
            <<Compute as ServiceMarker>::Provider as ProviderMarker>::ID,
            Example::ID
        );
    }

    #[test]
    fn accepts_canonical_bounded_identifiers() {
        assert_eq!(
            ProviderId::new("provider-9").map(ProviderId::as_str),
            Ok("provider-9")
        );
        assert_eq!(
            ServiceId::new("object-storage").map(ServiceId::as_str),
            Ok("object-storage")
        );
        assert!(ProviderId::new("a").is_ok());
    }

    #[test]
    fn rejects_every_noncanonical_identifier_class() {
        for value in [
            "",
            "-cloud",
            "cloud-",
            "cloud--api",
            "Cloud",
            "cloud_api",
            "c loud",
            "é",
        ] {
            assert!(ProviderId::new(value).is_err(), "{value}");
            assert!(ServiceId::new(value).is_err(), "{value}");
        }
        assert_eq!(ProviderId::new(""), Err(IdentityError::Empty));
        assert_eq!(
            ProviderId::new("cloud--api"),
            Err(IdentityError::RepeatedSeparator)
        );
        assert_eq!(
            ServiceId::new("-storage"),
            Err(IdentityError::InvalidBoundary)
        );
    }

    #[test]
    fn enforces_domain_length_limits_before_scanning() {
        const PROVIDER_MAX: &str =
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        const PROVIDER_LONG: &str =
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        const SERVICE_MAX: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        const SERVICE_LONG: &str =
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        assert_eq!(PROVIDER_MAX.len(), MAX_PROVIDER_ID_BYTES);
        assert_eq!(SERVICE_MAX.len(), MAX_SERVICE_ID_BYTES);
        assert!(ProviderId::new(PROVIDER_MAX).is_ok());
        assert!(ServiceId::new(SERVICE_MAX).is_ok());
        assert_eq!(ProviderId::new(PROVIDER_LONG), Err(IdentityError::TooLong));
        assert_eq!(ServiceId::new(SERVICE_LONG), Err(IdentityError::TooLong));
    }
}
