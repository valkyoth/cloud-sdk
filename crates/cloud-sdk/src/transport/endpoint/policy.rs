//! Provider-owned credential-destination policies.

use super::EndpointIdentity;

/// Maximum identities in one finite official endpoint set.
pub const MAX_OFFICIAL_ENDPOINTS: usize = 32;

/// Maximum provider region identifier length.
pub const MAX_ENDPOINT_REGION_BYTES: usize = 63;

/// Endpoint trust policy validation or matching error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointPolicyError {
    /// A finite official endpoint set must not be empty.
    EmptyOfficialSet,
    /// A finite official endpoint set exceeds [`MAX_OFFICIAL_ENDPOINTS`].
    TooManyOfficialEndpoints,
    /// A region identifier is not bounded canonical lowercase ASCII.
    InvalidRegion,
    /// The candidate endpoint is not admitted by the policy.
    DestinationMismatch,
}

impl_static_error!(EndpointPolicyError,
    Self::EmptyOfficialSet => "official endpoint set is empty",
    Self::TooManyOfficialEndpoints => "official endpoint set exceeds the length limit",
    Self::InvalidRegion => "endpoint region identifier is invalid",
    Self::DestinationMismatch => "endpoint destination is not admitted by policy",
);

/// Trust provenance represented by an [`EndpointPolicy`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EndpointPolicyKind {
    /// One provider-owned immutable endpoint.
    Fixed,
    /// One endpoint from a bounded provider-owned official set.
    OfficialSet,
    /// One endpoint derived by a provider from a validated region.
    RegionDerived,
    /// One custom endpoint explicitly acknowledged by trusted configuration.
    AcknowledgedCustom,
}

/// Explicit acknowledgement that a custom endpoint is a credential destination.
///
/// Construct this only at a trusted operator-configuration boundary. Tenant,
/// request, webhook, or other attacker-controlled input must never select the
/// endpoint. The type makes custom trust conspicuous but cannot determine
/// whether configuration is operationally trustworthy.
///
/// ```compile_fail
/// use cloud_sdk::transport::CustomEndpointAcknowledgement;
///
/// let acknowledgement = CustomEndpointAcknowledgement { private: () };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CustomEndpointAcknowledgement {
    private: (),
}

impl CustomEndpointAcknowledgement {
    /// Acknowledges a custom endpoint selected by trusted operator configuration.
    #[must_use]
    pub const fn trusted_operator_configuration() -> Self {
        Self { private: () }
    }
}

/// Custom endpoint paired with an explicit operator acknowledgement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcknowledgedCustomEndpoint<'a> {
    identity: EndpointIdentity<'a>,
    acknowledgement: CustomEndpointAcknowledgement,
}

impl<'a> AcknowledgedCustomEndpoint<'a> {
    /// Binds an endpoint identity to explicit operator acknowledgement.
    #[must_use]
    pub const fn new(
        identity: EndpointIdentity<'a>,
        acknowledgement: CustomEndpointAcknowledgement,
    ) -> Self {
        Self {
            identity,
            acknowledgement,
        }
    }

    /// Returns the acknowledged endpoint identity.
    #[must_use]
    pub const fn identity(self) -> EndpointIdentity<'a> {
        self.identity
    }
}

/// Provider-derived endpoint paired with its canonical region identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegionEndpoint<'a> {
    region: &'a str,
    identity: EndpointIdentity<'a>,
}

impl<'a> RegionEndpoint<'a> {
    /// Validates a provider region and its already-derived endpoint identity.
    pub fn new(
        region: &'a str,
        identity: EndpointIdentity<'a>,
    ) -> Result<Self, EndpointPolicyError> {
        validate_region(region)?;
        Ok(Self { region, identity })
    }

    /// Returns the provider region used for derivation.
    #[must_use]
    pub const fn region(self) -> &'a str {
        self.region
    }

    /// Returns the provider-derived endpoint identity.
    #[must_use]
    pub const fn identity(self) -> EndpointIdentity<'a> {
        self.identity
    }
}

/// Allocation-free provider-owned endpoint admission policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointPolicy<'a> {
    /// One immutable provider endpoint.
    Fixed(EndpointIdentity<'a>),
    /// A bounded finite set of official provider endpoints.
    OfficialSet(&'a [EndpointIdentity<'a>]),
    /// One endpoint derived from a provider region.
    RegionDerived(RegionEndpoint<'a>),
    /// One explicitly acknowledged custom credential destination.
    AcknowledgedCustom(AcknowledgedCustomEndpoint<'a>),
}

impl<'a> EndpointPolicy<'a> {
    /// Admits one fixed provider-owned endpoint.
    #[must_use]
    pub const fn fixed(identity: EndpointIdentity<'a>) -> Self {
        Self::Fixed(identity)
    }

    /// Admits one of a bounded finite provider-owned endpoint set.
    pub fn official_set(
        identities: &'a [EndpointIdentity<'a>],
    ) -> Result<Self, EndpointPolicyError> {
        if identities.is_empty() {
            return Err(EndpointPolicyError::EmptyOfficialSet);
        }
        if identities.len() > MAX_OFFICIAL_ENDPOINTS {
            return Err(EndpointPolicyError::TooManyOfficialEndpoints);
        }
        Ok(Self::OfficialSet(identities))
    }

    /// Admits one provider-derived regional endpoint.
    #[must_use]
    pub const fn region_derived(endpoint: RegionEndpoint<'a>) -> Self {
        Self::RegionDerived(endpoint)
    }

    /// Admits one explicitly acknowledged custom credential destination.
    #[must_use]
    pub const fn acknowledged_custom(endpoint: AcknowledgedCustomEndpoint<'a>) -> Self {
        Self::AcknowledgedCustom(endpoint)
    }

    /// Returns the policy's trust provenance.
    #[must_use]
    pub const fn kind(self) -> EndpointPolicyKind {
        match self {
            Self::Fixed(_) => EndpointPolicyKind::Fixed,
            Self::OfficialSet(_) => EndpointPolicyKind::OfficialSet,
            Self::RegionDerived(_) => EndpointPolicyKind::RegionDerived,
            Self::AcknowledgedCustom(_) => EndpointPolicyKind::AcknowledgedCustom,
        }
    }

    /// Reports whether the exact destination is admitted.
    #[must_use]
    pub fn admits(self, candidate: EndpointIdentity<'_>) -> bool {
        match self {
            Self::Fixed(identity) => identity == candidate,
            Self::OfficialSet(identities) => identities.contains(&candidate),
            Self::RegionDerived(endpoint) => endpoint.identity() == candidate,
            Self::AcknowledgedCustom(endpoint) => endpoint.identity() == candidate,
        }
    }

    /// Fails closed unless the exact destination is admitted.
    pub fn verify(self, candidate: EndpointIdentity<'_>) -> Result<(), EndpointPolicyError> {
        if self.admits(candidate) {
            Ok(())
        } else {
            Err(EndpointPolicyError::DestinationMismatch)
        }
    }
}

pub(super) fn validate_region(region: &str) -> Result<(), EndpointPolicyError> {
    if region.is_empty()
        || region.len() > MAX_ENDPOINT_REGION_BYTES
        || region.starts_with('-')
        || region.ends_with('-')
        || !region
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(EndpointPolicyError::InvalidRegion);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        AcknowledgedCustomEndpoint, CustomEndpointAcknowledgement, EndpointPolicy,
        EndpointPolicyError, EndpointPolicyKind, RegionEndpoint,
    };
    use crate::transport::{EndpointIdentity, EndpointScheme};

    #[test]
    fn all_policy_classes_match_only_their_exact_destination() {
        let a = identity("a.example");
        let b = identity("b.example");
        let other = identity("other.example");
        assert!(a.is_ok() && b.is_ok() && other.is_ok());
        let (Ok(a), Ok(b), Ok(other)) = (a, b, other) else {
            unreachable!("endpoint-policy fixture construction failed");
        };
        let official = [a, b];
        let set = EndpointPolicy::official_set(&official);
        let region = RegionEndpoint::new("eu-west-1", b);
        let custom = AcknowledgedCustomEndpoint::new(
            a,
            CustomEndpointAcknowledgement::trusted_operator_configuration(),
        );
        assert!(set.is_ok() && region.is_ok());
        if let (Ok(set), Ok(region)) = (set, region) {
            let policies = [
                EndpointPolicy::fixed(a),
                set,
                EndpointPolicy::region_derived(region),
                EndpointPolicy::acknowledged_custom(custom),
            ];
            let mut policies = policies.into_iter();
            assert!(policies.next().is_some_and(|policy| policy.kind()
                == EndpointPolicyKind::Fixed
                && policy.admits(a)));
            assert!(
                policies
                    .next()
                    .is_some_and(|policy| policy.admits(a) && policy.admits(b))
            );
            assert!(policies.next().is_some_and(|policy| policy.admits(b)));
            assert!(policies.next().is_some_and(|policy| policy.admits(a)));
            for policy in [
                EndpointPolicy::fixed(a),
                set,
                EndpointPolicy::region_derived(region),
                EndpointPolicy::acknowledged_custom(custom),
            ] {
                assert_eq!(
                    policy.verify(other),
                    Err(EndpointPolicyError::DestinationMismatch)
                );
            }
        } else {
            unreachable!("endpoint-policy fixture construction failed");
        }
    }

    #[test]
    fn official_sets_and_regions_are_bounded_and_canonical() {
        assert_eq!(
            EndpointPolicy::official_set(&[]),
            Err(EndpointPolicyError::EmptyOfficialSet)
        );
        let endpoint = identity("region.example");
        assert!(endpoint.is_ok());
        let Ok(endpoint) = endpoint else {
            unreachable!("endpoint-policy fixture construction failed");
        };
        for region in ["", "EU-west-1", "-eu", "eu-", "eu_west"] {
            assert_eq!(
                RegionEndpoint::new(region, endpoint),
                Err(EndpointPolicyError::InvalidRegion)
            );
        }
    }

    #[test]
    fn policy_accepts_non_static_provider_owned_identity() {
        let host_bytes = *b"runtime-region.example";
        let host = core::str::from_utf8(&host_bytes);
        assert!(host.is_ok());
        let Ok(host) = host else {
            unreachable!("endpoint-policy host fixture construction failed");
        };
        let endpoint = identity(host);
        assert!(endpoint.is_ok());
        if let Ok(endpoint) = endpoint {
            let regional = RegionEndpoint::new("runtime-1", endpoint);
            assert!(regional.is_ok());
            if let Ok(regional) = regional {
                assert!(EndpointPolicy::region_derived(regional).admits(endpoint));
            } else {
                unreachable!("regional endpoint-policy fixture construction failed");
            }
        } else {
            unreachable!("endpoint-policy fixture construction failed");
        }
    }

    fn identity(
        host: &str,
    ) -> Result<EndpointIdentity<'_>, crate::transport::EndpointIdentityError> {
        EndpointIdentity::new(EndpointScheme::Https, host, 443, "/v1")
    }
}
