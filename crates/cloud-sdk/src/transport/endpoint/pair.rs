use super::policy::validate_region;
use super::{EndpointIdentity, EndpointPolicyError, EndpointScheme};

/// Maximum regional API/token pairs in one provider policy.
pub const MAX_REGIONAL_ENDPOINT_PAIRS: usize = 32;

/// Regional endpoint-pair construction or matching failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointPairPolicyError {
    /// A regional pair policy must contain at least one pair.
    Empty,
    /// The pair set exceeds [`MAX_REGIONAL_ENDPOINT_PAIRS`].
    TooManyPairs,
    /// A region identifier is invalid.
    InvalidRegion,
    /// API and token endpoints must both use HTTPS.
    InsecureEndpoint,
    /// Regions and complete pairs must be unique.
    DuplicatePair,
    /// The supplied region, API endpoint, and token endpoint are not one pair.
    PairMismatch,
}

impl_static_error!(EndpointPairPolicyError,
    Self::Empty => "regional endpoint pair set is empty",
    Self::TooManyPairs => "regional endpoint pair set exceeds the length limit",
    Self::InvalidRegion => "regional endpoint pair has an invalid region",
    Self::InsecureEndpoint => "regional endpoint pair is not entirely HTTPS",
    Self::DuplicatePair => "regional endpoint pair set contains a duplicate",
    Self::PairMismatch => "API and token endpoints do not match one regional pair",
);

/// One exact provider-owned regional API and token-authority pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegionalEndpointPair<'a> {
    region: &'a str,
    api: EndpointIdentity<'a>,
    token: EndpointIdentity<'a>,
}

impl<'a> RegionalEndpointPair<'a> {
    /// Validates one exact HTTPS API/token endpoint pair.
    pub fn new(
        region: &'a str,
        api: EndpointIdentity<'a>,
        token: EndpointIdentity<'a>,
    ) -> Result<Self, EndpointPairPolicyError> {
        validate_region(region).map_err(map_region_error)?;
        if api.scheme() != EndpointScheme::Https || token.scheme() != EndpointScheme::Https {
            return Err(EndpointPairPolicyError::InsecureEndpoint);
        }
        Ok(Self { region, api, token })
    }

    /// Returns the canonical provider region.
    #[must_use]
    pub const fn region(self) -> &'a str {
        self.region
    }

    /// Returns the exact API endpoint identity.
    #[must_use]
    pub const fn api(self) -> EndpointIdentity<'a> {
        self.api
    }

    /// Returns the exact token endpoint identity.
    #[must_use]
    pub const fn token(self) -> EndpointIdentity<'a> {
        self.token
    }
}

/// Allocation-free finite policy for geographic API/token authority pairs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EndpointPairPolicy<'a> {
    pairs: &'a [RegionalEndpointPair<'a>],
}

impl<'a> EndpointPairPolicy<'a> {
    /// Validates a finite, unique provider-owned pair set.
    pub fn new(pairs: &'a [RegionalEndpointPair<'a>]) -> Result<Self, EndpointPairPolicyError> {
        if pairs.is_empty() {
            return Err(EndpointPairPolicyError::Empty);
        }
        if pairs.len() > MAX_REGIONAL_ENDPOINT_PAIRS {
            return Err(EndpointPairPolicyError::TooManyPairs);
        }
        for (index, pair) in pairs.iter().enumerate() {
            let Some(tail) = pairs.get(index.saturating_add(1)..) else {
                return Err(EndpointPairPolicyError::DuplicatePair);
            };
            if tail.iter().any(|other| {
                pair.region == other.region || pair.api == other.api || pair.token == other.token
            }) {
                return Err(EndpointPairPolicyError::DuplicatePair);
            }
        }
        Ok(Self { pairs })
    }

    /// Returns whether one exact regional API/token combination is admitted.
    #[must_use]
    pub fn admits(
        self,
        region: &str,
        api: EndpointIdentity<'_>,
        token: EndpointIdentity<'_>,
    ) -> bool {
        self.pairs
            .iter()
            .any(|pair| pair.region == region && pair.api == api && pair.token == token)
    }

    /// Fails closed unless all three values identify one reviewed pair.
    pub fn verify(
        self,
        region: &str,
        api: EndpointIdentity<'_>,
        token: EndpointIdentity<'_>,
    ) -> Result<(), EndpointPairPolicyError> {
        validate_region(region).map_err(map_region_error)?;
        if self.admits(region, api, token) {
            Ok(())
        } else {
            Err(EndpointPairPolicyError::PairMismatch)
        }
    }
}

const fn map_region_error(_error: EndpointPolicyError) -> EndpointPairPolicyError {
    EndpointPairPolicyError::InvalidRegion
}

#[cfg(test)]
mod tests {
    use super::{EndpointPairPolicy, EndpointPairPolicyError, RegionalEndpointPair};
    use crate::transport::{EndpointIdentity, EndpointScheme};

    fn endpoint(host: &'static str, path: &'static str) -> EndpointIdentity<'static> {
        EndpointIdentity::new(EndpointScheme::Https, host, 443, path)
            .unwrap_or_else(|_| unreachable!())
    }

    #[test]
    fn exact_region_pairs_reject_cross_region_and_alias_combinations() {
        let eu_api = endpoint("eu.api.example", "/v2");
        let eu_token = endpoint("www.example", "/auth/oauth2/token");
        let ca_api = endpoint("ca.api.example", "/v2");
        let ca_token = endpoint("ca.example", "/auth/oauth2/token");
        let pairs = [
            RegionalEndpointPair::new("eu", eu_api, eu_token).unwrap_or_else(|_| unreachable!()),
            RegionalEndpointPair::new("ca", ca_api, ca_token).unwrap_or_else(|_| unreachable!()),
        ];
        let policy = EndpointPairPolicy::new(&pairs).unwrap_or_else(|_| unreachable!());
        assert!(policy.verify("eu", eu_api, eu_token).is_ok());
        assert!(policy.verify("ca", ca_api, ca_token).is_ok());
        assert_eq!(
            policy.verify("eu", eu_api, ca_token),
            Err(EndpointPairPolicyError::PairMismatch)
        );
        assert_eq!(
            policy.verify("eu", endpoint("api.eu.example", "/v2"), eu_token,),
            Err(EndpointPairPolicyError::PairMismatch)
        );
    }

    #[test]
    fn pair_sets_are_nonempty_bounded_unique_and_https_only() {
        assert_eq!(
            EndpointPairPolicy::new(&[]),
            Err(EndpointPairPolicyError::Empty)
        );
        let api = endpoint("eu.api.example", "/v2");
        let token = endpoint("www.example", "/auth/oauth2/token");
        let pair = RegionalEndpointPair::new("eu", api, token).unwrap_or_else(|_| unreachable!());
        assert_eq!(
            EndpointPairPolicy::new(&[pair, pair]),
            Err(EndpointPairPolicyError::DuplicatePair)
        );
        let http = EndpointIdentity::new(EndpointScheme::Http, "www.example", 80, "/token")
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(
            RegionalEndpointPair::new("eu", api, http),
            Err(EndpointPairPolicyError::InsecureEndpoint)
        );
    }
}
