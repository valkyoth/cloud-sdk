//! Provider service identity and endpoint trust binding.

use crate::transport::EndpointPolicy;
use crate::{ProviderId, ProviderMarker, ServiceId, ServiceMarker};

/// Provider service and immutable endpoint trust policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderService<'endpoint> {
    provider_id: ProviderId,
    service_id: ServiceId,
    endpoint_policy: EndpointPolicy<'endpoint>,
}

impl<'endpoint> ProviderService<'endpoint> {
    /// Binds validated provider and service IDs to an endpoint trust policy.
    #[must_use]
    pub const fn new(
        provider_id: ProviderId,
        service_id: ServiceId,
        endpoint_policy: EndpointPolicy<'endpoint>,
    ) -> Self {
        Self {
            provider_id,
            service_id,
            endpoint_policy,
        }
    }

    /// Binds a provider-owned service marker to an endpoint trust policy.
    #[must_use]
    pub const fn from_marker<S: ServiceMarker>(endpoint_policy: EndpointPolicy<'endpoint>) -> Self {
        Self::new(<S::Provider as ProviderMarker>::ID, S::ID, endpoint_policy)
    }

    /// Returns the canonical provider namespace.
    #[must_use]
    pub const fn provider_id(self) -> ProviderId {
        self.provider_id
    }

    /// Returns the canonical provider-owned service namespace.
    #[must_use]
    pub const fn service_id(self) -> ServiceId {
        self.service_id
    }

    /// Returns the immutable endpoint trust policy.
    #[must_use]
    pub const fn endpoint_policy(self) -> EndpointPolicy<'endpoint> {
        self.endpoint_policy
    }
}
