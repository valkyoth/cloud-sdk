use crate::authentication::AuthenticationScopePolicy;
use crate::operation::{OperationImpact, OperationMetadata, RequestIdPolicy, ResponsePolicy};
use crate::transport::{RawResponsePolicy, TransportRequest};

use super::{
    ApprovedReadOnlyPostQuery, BodyReplayability, PreparedRequest, PreparedRequestPolicyError,
    ProviderService, RequestBodySensitivity,
};

impl<'request> PreparedRequest<'request> {
    /// Creates a complete prepared request after checking cross-policy invariants.
    ///
    /// # Errors
    ///
    /// Returns [`PreparedRequestPolicyError::MissingRequestIdHeader`] when
    /// operation metadata protects or retains request IDs but the raw response
    /// policy does not admit `x-request-id`. Returns
    /// [`PreparedRequestPolicyError::ReadOnlyMethodMismatch`] when read-only
    /// metadata is paired with any method other than `GET` or `HEAD`.
    /// `body_sensitivity` is mandatory so provider implementations cannot omit
    /// the confidential-body review and silently receive a public default.
    pub fn new(
        request: TransportRequest<'request>,
        service: ProviderService<'request>,
        metadata: OperationMetadata,
        response_policy: ResponsePolicy,
        authentication_policy: AuthenticationScopePolicy<'request>,
        raw_response_policy: RawResponsePolicy<'request>,
        body_sensitivity: RequestBodySensitivity,
    ) -> Result<Self, PreparedRequestPolicyError> {
        Self::new_inner(
            request,
            service,
            metadata,
            response_policy,
            authentication_policy,
            &raw_response_policy,
            body_sensitivity,
            false,
        )
    }

    /// Creates a registry-approved read-only query carried by `POST`.
    ///
    /// The closed approval entry validates provider, service, official
    /// endpoint, operation ID, method, target, headers, body presence,
    /// authentication scope, and complete safety metadata before permitless
    /// execution is admitted.
    #[allow(clippy::too_many_arguments)]
    pub fn new_read_only_post_query(
        approval: ApprovedReadOnlyPostQuery,
        request: TransportRequest<'request>,
        service: ProviderService<'request>,
        metadata: OperationMetadata,
        response_policy: ResponsePolicy,
        authentication_policy: AuthenticationScopePolicy<'request>,
        raw_response_policy: RawResponsePolicy<'request>,
        body_sensitivity: RequestBodySensitivity,
    ) -> Result<Self, PreparedRequestPolicyError> {
        let operation_id = approval
            .validate(request, service, metadata, authentication_policy)
            .ok_or(PreparedRequestPolicyError::ReadOnlyPostQueryMismatch)?;
        let mut prepared = Self::new_inner(
            request,
            service,
            metadata,
            response_policy,
            authentication_policy,
            &raw_response_policy,
            body_sensitivity,
            true,
        )?;
        prepared.operation_id = Some(operation_id);
        Ok(prepared)
    }

    #[allow(clippy::too_many_arguments)]
    fn new_inner(
        request: TransportRequest<'request>,
        service: ProviderService<'request>,
        metadata: OperationMetadata,
        response_policy: ResponsePolicy,
        authentication_policy: AuthenticationScopePolicy<'request>,
        raw_response_policy: &RawResponsePolicy<'request>,
        body_sensitivity: RequestBodySensitivity,
        approved_read_only_post: bool,
    ) -> Result<Self, PreparedRequestPolicyError> {
        if matches!(metadata.impact(), OperationImpact::ReadOnly)
            && !request.method().permits_direct_read_only()
            && !approved_read_only_post
        {
            return Err(PreparedRequestPolicyError::ReadOnlyMethodMismatch);
        }
        if metadata.request_id_policy() != RequestIdPolicy::Discard
            && !raw_response_policy.admits_header("x-request-id")
        {
            return Err(PreparedRequestPolicyError::MissingRequestIdHeader);
        }
        Ok(Self {
            request,
            service,
            metadata,
            response_policy,
            authentication_policy,
            raw_response_policy: *raw_response_policy,
            operation_id: None,
            body_replayability: if request.body().is_empty() {
                BodyReplayability::Replayable
            } else {
                BodyReplayability::NotReplayable
            },
            body_sensitivity,
            authorization_evidence_required: false,
            approved_read_only_post,
        })
    }
}
