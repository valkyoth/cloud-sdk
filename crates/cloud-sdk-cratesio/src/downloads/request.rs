use crate::{
    endpoint::ApiRequestTarget,
    identifiers::{CrateName, Version},
    query::{
        ApiPath, FixedSegment as F, Parameter, PathSegment as P, Query, QueryError, QueryOperation,
    },
};
use cloud_sdk::operation::{
    OperationId, OperationIdError, OperationMetadata, OperationMetadataError,
};

/// Source-owned anonymous GET operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DownloadOperation {
    /// JSON location profile, not the binary archive.
    Location,
    /// Latest five versions plus aggregated older-version counts.
    CrateCounts,
    /// Inclusive ninety-day version count window.
    VersionCounts,
    /// Numbered reverse dependency page, not a dependency resolver.
    ReverseDependencies,
}
impl DownloadOperation {
    /// Exact source identifier.
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::Location => "download_version",
            Self::CrateCounts => "get_crate_downloads",
            Self::VersionCounts => "get_version_downloads",
            Self::ReverseDependencies => "list_reverse_dependencies",
        }
    }
    /// Checked source identifier.
    pub fn operation_id(self) -> Result<OperationId, OperationIdError> {
        OperationId::new(self.operation_name())
    }
    /// Read-only, with no implicit retry.
    pub const fn metadata(self) -> Result<OperationMetadata, OperationMetadataError> {
        crate::discovery::DiscoveryOperation::Summary.metadata()
    }
}
/// Immutable bounded download and usage request.
#[derive(Clone, Copy, Debug)]
pub struct DownloadRequest<'a> {
    pub(super) operation: DownloadOperation,
    pub(super) name: CrateName<'a>,
    pub(super) version: Option<Version<'a>>,
    pub(super) query: Option<Query<'a>>,
}
impl<'a> DownloadRequest<'a> {
    /// Requests JSON location without following redirects or sending credentials.
    pub const fn location(name: CrateName<'a>, version: Version<'a>) -> Self {
        Self {
            operation: DownloadOperation::Location,
            name,
            version: Some(version),
            query: None,
        }
    }
    /// Crate download counts with optional `include=versions`.
    pub fn crate_counts(
        name: CrateName<'a>,
        parameters: &'a [Parameter<'a>],
    ) -> Result<Self, QueryError> {
        Ok(Self {
            operation: DownloadOperation::CrateCounts,
            name,
            version: None,
            query: Some(Query::new(QueryOperation::Downloads, parameters)?),
        })
    }
    /// Version counts with an optional inclusive `before_date`.
    pub fn version_counts(
        name: CrateName<'a>,
        version: Version<'a>,
        parameters: &'a [Parameter<'a>],
    ) -> Result<Self, QueryError> {
        Ok(Self {
            operation: DownloadOperation::VersionCounts,
            name,
            version: Some(version),
            query: Some(Query::new(QueryOperation::VersionDownloads, parameters)?),
        })
    }
    /// Explicit bounded numbered page. Seek is not implemented by this controller.
    pub fn reverse_dependencies(
        name: CrateName<'a>,
        parameters: &'a [Parameter<'a>],
    ) -> Result<Self, QueryError> {
        if !parameters
            .iter()
            .any(|p| matches!(p, Parameter::PerPage(_)))
            || parameters.iter().any(|p| matches!(p, Parameter::Seek(_)))
        {
            return Err(QueryError::Operation);
        }
        Ok(Self {
            operation: DownloadOperation::ReverseDependencies,
            name,
            version: None,
            query: Some(Query::new(QueryOperation::ReverseDependencies, parameters)?),
        })
    }
    /// Immutable operation.
    pub const fn operation(self) -> DownloadOperation {
        self.operation
    }
    /// Validated selectors, if any.
    pub const fn query(self) -> Option<Query<'a>> {
        self.query
    }
    /// Atomic caller-buffer target encoding.
    pub fn write_target(self, output: &mut [u8]) -> Result<ApiRequestTarget<'_>, QueryError> {
        let last = match self.operation {
            DownloadOperation::Location => F::Download,
            DownloadOperation::ReverseDependencies => F::ReverseDependencies,
            _ => F::Downloads,
        };
        let parts = [
            P::Fixed(F::Crates),
            P::Crate(self.name),
            self.version.map(P::Version).unwrap_or(P::Fixed(last)),
            P::Fixed(last),
        ];
        let path = ApiPath::new(
            parts
                .get(..if self.version.is_some() { 4 } else { 3 })
                .ok_or(QueryError::Output)?,
        )?;
        match self.query {
            Some(query) => query.write_target(path, output),
            None => path.write(output),
        }
    }
}
