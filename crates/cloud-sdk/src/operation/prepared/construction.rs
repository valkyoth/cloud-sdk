use crate::Method;
use crate::authentication::AuthenticationScopePolicy;
use crate::operation::{OperationImpact, OperationMetadata, RequestIdPolicy, ResponsePolicy};
use crate::transport::{RawResponsePolicy, TransportRequest};

use super::{
    BodyReplayability, PreparedRequest, PreparedRequestPolicyError, ProviderService,
    RequestBodySensitivity,
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

    /// Creates an explicitly reviewed read-only query carried by `POST`.
    ///
    /// This narrow constructor exists for provider protocols that use a form
    /// body to describe a query. It rejects methods other than `POST` and
    /// metadata other than read-only/safe. The resulting request can execute
    /// without mutation authority, while retry still follows its explicit
    /// metadata and body-replayability policies.
    pub fn new_read_only_post_query(
        request: TransportRequest<'request>,
        service: ProviderService<'request>,
        metadata: OperationMetadata,
        response_policy: ResponsePolicy,
        authentication_policy: AuthenticationScopePolicy<'request>,
        raw_response_policy: RawResponsePolicy<'request>,
        body_sensitivity: RequestBodySensitivity,
    ) -> Result<Self, PreparedRequestPolicyError> {
        if request.method() != Method::Post
            || !matches!(metadata.impact(), OperationImpact::ReadOnly)
        {
            return Err(PreparedRequestPolicyError::ReadOnlyPostQueryMismatch);
        }
        Self::new_inner(
            request,
            service,
            metadata,
            response_policy,
            authentication_policy,
            &raw_response_policy,
            body_sensitivity,
            true,
        )
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
        direct_read_only_post: bool,
    ) -> Result<Self, PreparedRequestPolicyError> {
        if matches!(metadata.impact(), OperationImpact::ReadOnly)
            && !request.method().permits_direct_read_only()
            && !direct_read_only_post
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
            direct_read_only_post,
        })
    }
}
