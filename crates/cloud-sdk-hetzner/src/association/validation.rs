//! Canonical association and prepared-wire policy validation.

use cloud_sdk::Method;
#[cfg(test)]
use cloud_sdk::operation::PreparedRequest;
use cloud_sdk::operation::{
    BodyReplayability, CostIntent, OperationId, OperationImpact, OperationMetadata,
    ProviderService, RequestIdPolicy, RequestSemantics, ResponsePolicy, RetryEligibility,
};
#[cfg(test)]
use cloud_sdk::transport::{MediaType, TransportRequest};
use cloud_sdk::transport::{RawResponsePolicy, StatusCode};

use super::components::AssociationError;
use super::policy::{
    AuthenticationClass, BodyPolicy, HetznerOperation, OperationDescriptor, PermitClass,
    QueryPolicy, ResponseShape, RetryPolicy,
};
use crate::endpoint::ApiSurface;
use crate::prepared::{
    EndpointWire, RequestShape, ResponseProfile, authentication_policy, provider_service,
    raw_response_policy, response_policy,
};

/// Complete association policy checked against one endpoint snapshot.
#[derive(Clone, Copy)]
pub(crate) struct ValidatedAssociationPolicy {
    pub(crate) operation_id: OperationId,
    pub(crate) method: Method,
    pub(crate) service: ProviderService<'static>,
    pub(crate) metadata: OperationMetadata,
    pub(crate) response: ResponsePolicy,
    pub(crate) authentication: cloud_sdk::authentication::AuthenticationScopePolicy<'static>,
    pub(crate) raw_response: RawResponsePolicy<'static>,
    pub(crate) body_replayability: BodyReplayability,
    pub(crate) profile: ResponseProfile,
    pub(crate) request_shape: RequestShape,
}

pub(crate) fn validate_association<O, E>(
    endpoint: E,
) -> Result<ValidatedAssociationPolicy, AssociationError>
where
    O: HetznerOperation,
    E: EndpointWire,
{
    let descriptor = O::DESCRIPTOR;
    let expected = canonical_policy(descriptor)?;
    let operation_key = endpoint.operation_key();
    let method = endpoint.method();
    let api_base_url = endpoint.api_base_url();
    let endpoint_group = endpoint.endpoint_group();
    let request_shape = endpoint.request_shape();
    let response_profile = endpoint.response_profile();
    let metadata = endpoint.metadata().map_err(policy_mismatch)?;
    let runtime_service = provider_service(endpoint_group).map_err(policy_mismatch)?;
    let runtime_authentication =
        authentication_policy(runtime_service, api_base_url).map_err(policy_mismatch)?;
    let runtime_response = response_policy(response_profile).map_err(policy_mismatch)?;
    let runtime_raw = raw_response_policy(response_profile).map_err(policy_mismatch)?;
    let authentication = match endpoint_group.surface() {
        ApiSurface::Storage => AuthenticationClass::Basic,
        ApiSurface::Cloud | ApiSurface::Dns | ApiSurface::Security => AuthenticationClass::Bearer,
    };

    if operation_key != expected.operation_id.as_str()
        || method != expected.method
        || api_base_url != descriptor.api_base_url()
        || authentication != descriptor.authentication()
        || request_shape != expected.request_shape
        || response_profile != expected.profile
        || metadata != expected.metadata
        || runtime_service != expected.service
        || runtime_authentication != expected.authentication
        || runtime_response != expected.response
        || runtime_raw != expected.raw_response
    {
        return Err(AssociationError::PreparedPolicyMismatch);
    }
    Ok(expected)
}

#[cfg(test)]
pub(crate) fn prepared_policy_matches<O: HetznerOperation>(
    prepared: &PreparedRequest<'_>,
) -> Result<(), AssociationError> {
    let descriptor = O::DESCRIPTOR;
    let expected = canonical_policy(descriptor)?;
    let request = prepared.transport_request();
    if prepared.operation_id() != Some(expected.operation_id)
        || request.method() != expected.method
        || prepared.service() != expected.service
        || prepared.metadata() != expected.metadata
        || prepared.response_policy() != expected.response
        || prepared.authentication_policy() != expected.authentication
        || prepared.raw_response_policy() != expected.raw_response
        || prepared.body_replayability() != expected.body_replayability
        || !request_shape_matches(descriptor, request)
        || !request_headers_match(descriptor, request)
    {
        return Err(AssociationError::PreparedPolicyMismatch);
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn descriptor_policy_is_coherent(descriptor: OperationDescriptor) -> bool {
    canonical_policy(descriptor).is_ok()
}

fn canonical_policy(
    descriptor: OperationDescriptor,
) -> Result<ValidatedAssociationPolicy, AssociationError> {
    let service = provider_service_for(descriptor)?;
    let profile = response_profile_for(descriptor)?;
    Ok(ValidatedAssociationPolicy {
        operation_id: descriptor.operation_id(),
        method: descriptor.method(),
        service,
        metadata: metadata_for(descriptor)?,
        response: response_policy(profile).map_err(policy_mismatch)?,
        authentication: authentication_policy(service, descriptor.api_base_url())
            .map_err(policy_mismatch)?,
        raw_response: raw_response_policy(profile).map_err(policy_mismatch)?,
        body_replayability: BodyReplayability::Replayable,
        profile,
        request_shape: request_shape_for(descriptor)?,
    })
}

fn provider_service_for(
    descriptor: OperationDescriptor,
) -> Result<ProviderService<'static>, AssociationError> {
    let policy =
        crate::official_endpoint_policy(descriptor.api_base_url()).map_err(policy_mismatch)?;
    Ok(ProviderService::new(
        crate::HETZNER_PROVIDER_ID,
        descriptor.service_id(),
        policy,
    ))
}

fn metadata_for(descriptor: OperationDescriptor) -> Result<OperationMetadata, AssociationError> {
    let no_cost = CostIntent::NoKnownCost;
    let cost = CostIntent::MayIncurCost;
    let explicit = RetryEligibility::ExplicitPolicy;
    let never = RetryEligibility::Never;
    let (impact, semantics, retry, cost_intent) =
        match (descriptor.method(), descriptor.retry(), descriptor.permit()) {
            (cloud_sdk::Method::Get, RetryPolicy::Explicit, PermitClass::None) => (
                OperationImpact::ReadOnly,
                RequestSemantics::Safe,
                explicit,
                no_cost,
            ),
            (cloud_sdk::Method::Put, RetryPolicy::Explicit, PermitClass::Mutation) => (
                OperationImpact::Mutation,
                RequestSemantics::Idempotent,
                explicit,
                no_cost,
            ),
            (cloud_sdk::Method::Post, RetryPolicy::Never, PermitClass::Mutation) => (
                OperationImpact::Mutation,
                RequestSemantics::NonIdempotent,
                never,
                no_cost,
            ),
            (cloud_sdk::Method::Post, RetryPolicy::Never, PermitClass::Cost) => (
                OperationImpact::Mutation,
                RequestSemantics::NonIdempotent,
                never,
                cost,
            ),
            (cloud_sdk::Method::Delete, RetryPolicy::Never, PermitClass::Destructive) => (
                OperationImpact::Destructive,
                RequestSemantics::Idempotent,
                never,
                no_cost,
            ),
            (cloud_sdk::Method::Post, RetryPolicy::Never, PermitClass::Destructive) => (
                OperationImpact::Destructive,
                RequestSemantics::NonIdempotent,
                never,
                no_cost,
            ),
            _ => return Err(AssociationError::PreparedPolicyMismatch),
        };
    OperationMetadata::new(
        impact,
        semantics,
        retry,
        cost_intent,
        RequestIdPolicy::Protected,
    )
    .map_err(policy_mismatch)
}

fn response_profile_for(
    descriptor: OperationDescriptor,
) -> Result<ResponseProfile, AssociationError> {
    match (descriptor.success_status(), descriptor.response_shape()) {
        (StatusCode::NO_CONTENT, ResponseShape::Empty) => Ok(ResponseProfile::NoContent),
        (StatusCode::OK, shape) if !matches!(shape, ResponseShape::Empty) => {
            Ok(ResponseProfile::JsonOk)
        }
        (StatusCode::CREATED, shape) if !matches!(shape, ResponseShape::Empty) => {
            Ok(ResponseProfile::JsonCreated)
        }
        _ => Err(AssociationError::PreparedPolicyMismatch),
    }
}

fn request_shape_for(descriptor: OperationDescriptor) -> Result<RequestShape, AssociationError> {
    match (descriptor.query_policy(), descriptor.body_policy()) {
        (QueryPolicy::Forbidden, BodyPolicy::Forbidden) => Ok(RequestShape::None),
        (QueryPolicy::Optional, BodyPolicy::Forbidden) => Ok(RequestShape::OptionalQuery),
        (QueryPolicy::Required, BodyPolicy::Forbidden) => Ok(RequestShape::RequiredQuery),
        (QueryPolicy::Forbidden, BodyPolicy::RequiredJson) => Ok(RequestShape::RequiredJson),
        _ => Err(AssociationError::PreparedPolicyMismatch),
    }
}

#[cfg(test)]
fn request_shape_matches(descriptor: OperationDescriptor, request: TransportRequest<'_>) -> bool {
    let has_query = request.target().as_str().contains('?');
    let has_body = !request.body().is_empty();
    match (descriptor.query_policy(), descriptor.body_policy()) {
        (QueryPolicy::Forbidden, BodyPolicy::Forbidden) => !has_query && !has_body,
        (QueryPolicy::Optional, BodyPolicy::Forbidden) => !has_body,
        (QueryPolicy::Required, BodyPolicy::Forbidden) => has_query && !has_body,
        (QueryPolicy::Forbidden, BodyPolicy::RequiredJson) => !has_query && has_body,
        _ => false,
    }
}

#[cfg(test)]
fn request_headers_match(descriptor: OperationDescriptor, request: TransportRequest<'_>) -> bool {
    let headers = request.headers();
    let accept = headers
        .get("accept")
        .is_some_and(|header| header.value().as_str() == MediaType::JSON.as_str());
    let content_type = headers
        .get("content-type")
        .is_some_and(|header| header.value().as_str() == MediaType::JSON.as_str());
    match descriptor.body_policy() {
        BodyPolicy::Forbidden => accept && !content_type && headers.as_slice().len() == 1,
        BodyPolicy::RequiredJson => accept && content_type && headers.as_slice().len() == 2,
    }
}

fn policy_mismatch<T>(_: T) -> AssociationError {
    AssociationError::PreparedPolicyMismatch
}
