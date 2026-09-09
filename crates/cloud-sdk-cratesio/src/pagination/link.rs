use super::{Cursor, Direction, PaginationError, compare::check_query};
use crate::{
    endpoint::{OfficialCratesIoEndpoint, OfficialEndpointPurpose},
    query::{ApiPath, MAX_TARGET_BYTES, Query, QueryOperation},
};
use cloud_sdk::{
    buffer::encode_snapshot_bounded,
    operation::OperationId,
    pagination::{PaginationLimits, ProviderLinkBinding, ValidatedProviderLink},
    transport::BoundTransport,
};
use core::fmt;

/// Exact provider query bound to one official authority and immutable request.
/// No raw constructor, cloning, automatic execution, or credential application.
pub struct PageLink<'a> {
    endpoint: OfficialCratesIoEndpoint,
    path: ApiPath<'a>,
    query: &'a str,
    cursor: Cursor<'a>,
    operation: OperationId,
}
impl<'a> PageLink<'a> {
    /// Admits query-only, origin-form, or exact official absolute meta links.
    ///
    /// Every original filter, array entry, include, sort and page-size value
    /// must remain unchanged. Only page/seek may advance. Other origins,
    /// userinfo, explicit ports, fragments and path changes are rejected.
    pub fn new(
        endpoint: OfficialCratesIoEndpoint,
        path: ApiPath<'a>,
        current: Query<'a>,
        link: &'a str,
        direction: Direction,
    ) -> Result<Self, PaginationError> {
        if link.is_empty() || link.len() > MAX_TARGET_BYTES {
            return Err(PaginationError::Limit);
        }
        if endpoint.purpose() == OfficialEndpointPurpose::StaticDownloads
            || !current.operation().paginated()
            || !path.supports(current.operation())
        {
            return Err(PaginationError::Binding);
        }
        let mut path_bytes = [0; MAX_TARGET_BYTES];
        let expected_path = path
            .write(&mut path_bytes)
            .map_err(|_| PaginationError::Limit)?;
        let relative = if link.starts_with('?') || link.starts_with('/') {
            link
        } else {
            let relative = link
                .strip_prefix(endpoint.base_url())
                .ok_or(PaginationError::Binding)?;
            if !relative.starts_with('/') {
                return Err(PaginationError::Binding);
            }
            relative
        };
        let query = if let Some(q) = relative.strip_prefix('?') {
            q
        } else {
            let (p, q) = relative.split_once('?').ok_or(PaginationError::Invalid)?;
            if p != expected_path.as_str() {
                return Err(PaginationError::Binding);
            }
            q
        };
        let mut query_bytes = [0; MAX_TARGET_BYTES];
        let expected_query = current.write(&mut query_bytes)?;
        let cursor = check_query(query, expected_query, current, direction)?;
        let result = Self {
            endpoint,
            path,
            query,
            cursor,
            operation: operation_id(current.operation())?,
        };
        // Apply the neutral provider-link URI validator before exposing state.
        let mut checked = [0; MAX_TARGET_BYTES];
        let limits =
            PaginationLimits::new(1, 1, MAX_TARGET_BYTES).map_err(|_| PaginationError::Limit)?;
        result.transfer_to(&mut path_bytes, &mut checked, limits)?;
        Ok(result)
    }
    /// Returns the validated continuation kind without disclosing it in logs.
    #[must_use]
    pub const fn cursor(&self) -> Cursor<'a> {
        self.cursor
    }
    /// Returns the exact official origin binding.
    #[must_use]
    pub const fn endpoint(&self) -> OfficialCratesIoEndpoint {
        self.endpoint
    }
    /// Returns the source-owned operation required for continuation dispatch.
    #[must_use]
    pub const fn operation(&self) -> OperationId {
        self.operation
    }
    /// Verifies the transport that a caller intends to use. Actual clients must
    /// bind this check and dispatch; this method does not authorize credentials.
    pub fn verify_transport(
        &self,
        transport: &(impl BoundTransport + ?Sized),
    ) -> Result<(), PaginationError> {
        self.endpoint
            .verify_transport(transport)
            .map_err(|_| PaginationError::Binding)
    }
    /// Transfers exact query bytes into the core's cleanup-owning execution
    /// type. That type binds endpoint, GET method and operation at dispatch.
    /// Caller path storage remains borrowed for the returned link lifetime.
    pub fn transfer_to<'p, 'o>(
        &self,
        path_storage: &'p mut [u8],
        output: &'o mut [u8],
        limits: PaginationLimits,
    ) -> Result<ValidatedProviderLink<'o, 'p>, PaginationError> {
        let path = self.path.write(path_storage)?;
        let binding = ProviderLinkBinding::new(
            self.endpoint
                .identity()
                .map_err(|_| PaginationError::Binding)?,
            self.operation,
            path.as_request_target().path(),
        );
        let mut source = [0; MAX_TARGET_BYTES];
        let len = self.encode_target(&mut source)?;
        ValidatedProviderLink::transfer_from(
            source.get_mut(..len).ok_or(PaginationError::Limit)?,
            output,
            binding,
            limits,
        )
        .map_err(|_| PaginationError::Invalid)
    }
    pub(super) fn encode_target(&self, output: &mut [u8]) -> Result<usize, PaginationError> {
        let mut path = [0; MAX_TARGET_BYTES];
        let path = self.path.write(&mut path)?;
        let len = encode_snapshot_bounded(
            (path.as_str(), self.query),
            output,
            MAX_TARGET_BYTES,
            PaginationError::Limit,
            |(path, query), e| {
                e.string(path)?;
                e.byte(b'?')?;
                e.string(query)
            },
        )?;
        Ok(len)
    }
    pub(super) fn state_bytes(&self) -> usize {
        self.query.len()
    }
}
impl fmt::Debug for PageLink<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PageLink([redacted])")
    }
}

fn operation_id(op: QueryOperation) -> Result<OperationId, PaginationError> {
    let name = match op {
        QueryOperation::Crates => "list_crates",
        QueryOperation::Categories => "list_categories",
        QueryOperation::Keywords => "list_keywords",
        QueryOperation::Versions => "list_versions",
        QueryOperation::ReverseDependencies => "list_reverse_dependencies",
        QueryOperation::GithubConfigs => "list_trustpub_github_configs",
        QueryOperation::GitlabConfigs => "list_trustpub_gitlab_configs",
        _ => return Err(PaginationError::Binding),
    };
    OperationId::new(name).map_err(|_| PaginationError::Binding)
}
