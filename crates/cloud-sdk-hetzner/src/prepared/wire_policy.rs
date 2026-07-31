//! Provider-owned service, authentication, and response-wire policies.

use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::operation::ProviderService;
use cloud_sdk::transport::{
    HeaderName, MAX_INFORMATIONAL_RESPONSES, MediaType, RawResponsePolicy, ResponseMediaPolicy,
};

use crate::endpoint::{
    ApiSurface, EndpointGroup, official_endpoint_identity, official_endpoint_policy,
};
use crate::identity::{CloudService, DnsService, SecurityService, StorageService};
use crate::request::ApiBaseUrl;

use super::{HetznerPreparationError, ResponseProfile};

const JSON_MEDIA: &[MediaType<'static>] = &[MediaType::JSON];
const MAX_JSON_RESPONSE_BYTES: usize = 8_388_608;

pub(super) fn provider_service(
    group: EndpointGroup,
) -> Result<ProviderService<'static>, HetznerPreparationError> {
    let policy = official_endpoint_policy(group.api_base_url())
        .map_err(HetznerPreparationError::InvalidOfficialEndpoint)?;
    Ok(match group.surface() {
        ApiSurface::Cloud => ProviderService::from_marker::<CloudService>(policy),
        ApiSurface::Dns => ProviderService::from_marker::<DnsService>(policy),
        ApiSurface::Security => ProviderService::from_marker::<SecurityService>(policy),
        ApiSurface::Storage => ProviderService::from_marker::<StorageService>(policy),
    })
}

pub(super) fn authentication_policy(
    service: ProviderService<'static>,
    base: ApiBaseUrl,
) -> Result<AuthenticationScopePolicy<'static>, HetznerPreparationError> {
    let endpoint = official_endpoint_identity(base)
        .map_err(HetznerPreparationError::InvalidOfficialEndpoint)?;
    Ok(AuthenticationScopePolicy::new(
        ScopeRequirement::Required(service.provider_id()),
        ScopeRequirement::Required(service.service_id()),
        ScopeRequirement::Required(endpoint),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    ))
}

pub(super) fn raw_response_policy(
    profile: ResponseProfile,
) -> Result<RawResponsePolicy<'static>, HetznerPreparationError> {
    let content_type =
        HeaderName::new("content-type").map_err(HetznerPreparationError::InvalidHeaders)?;
    let (success_bytes, success_media) = match profile {
        ResponseProfile::JsonOk | ResponseProfile::JsonCreated => (
            MAX_JSON_RESPONSE_BYTES,
            ResponseMediaPolicy::Required(JSON_MEDIA),
        ),
        ResponseProfile::NoContent => (0, ResponseMediaPolicy::Forbidden),
    };
    RawResponsePolicy::new(
        success_bytes,
        MAX_JSON_RESPONSE_BYTES,
        success_media,
        ResponseMediaPolicy::Required(JSON_MEDIA),
        &[content_type],
        MAX_INFORMATIONAL_RESPONSES,
    )
    .map_err(HetznerPreparationError::InvalidRawResponsePolicy)
}
