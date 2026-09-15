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

/// Reviewed anonymous GET operations. Authors is deprecated upstream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VersionOperation {
    /// Paged version list, not an index mirror.
    List,
    /// Exact version detail.
    Detail,
    /// Dependency metadata, not a resolver.
    Dependencies,
    /// Deprecated empty compatibility response.
    Authors,
    /// JSON location of the static HTML README, not the HTML itself.
    Readme,
}
impl VersionOperation {
    /// Source-owned identifier.
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::List => "list_versions",
            Self::Detail => "find_version",
            Self::Dependencies => "get_version_dependencies",
            Self::Authors => "get_version_authors",
            Self::Readme => "get_version_readme",
        }
    }
    /// Typed source identifier.
    pub fn operation_id(self) -> Result<OperationId, OperationIdError> {
        OperationId::new(self.operation_name())
    }
    /// Read-only GET with retries disabled.
    pub const fn metadata(self) -> Result<OperationMetadata, OperationMetadataError> {
        crate::discovery::DiscoveryOperation::Summary.metadata()
    }
}
/// Immutable operation binding with allocation-free target construction.
#[derive(Clone, Copy, Debug)]
pub struct VersionRequest<'a> {
    pub(super) operation: VersionOperation,
    pub(super) name: CrateName<'a>,
    pub(super) version: Option<Version<'a>>,
    pub(super) query: Option<Query<'a>>,
}
impl<'a> VersionRequest<'a> {
    /// Checked version-list selectors, including explicit release tracks.
    /// Requires PerPage and rejects Page: upstream otherwise returns all versions
    /// and only supports seek pagination. No unpaginated client mode is exposed.
    pub fn list(name: CrateName<'a>, parameters: &'a [Parameter<'a>]) -> Result<Self, QueryError> {
        if !parameters
            .iter()
            .any(|p| matches!(p, Parameter::PerPage(_)))
            || parameters.iter().any(|p| matches!(p, Parameter::Page(_)))
        {
            return Err(QueryError::Operation);
        }
        Ok(Self {
            operation: VersionOperation::List,
            name,
            version: None,
            query: Some(Query::new(QueryOperation::Versions, parameters)?),
        })
    }
    fn exact(operation: VersionOperation, name: CrateName<'a>, version: Version<'a>) -> Self {
        Self {
            operation,
            name,
            version: Some(version),
            query: None,
        }
    }
    /// Exact version metadata, including build suffix identity.
    pub fn detail(name: CrateName<'a>, version: Version<'a>) -> Self {
        Self::exact(VersionOperation::Detail, name, version)
    }
    /// Version dependency records. Prefer the sparse index for resolution.
    pub fn dependencies(name: CrateName<'a>, version: Version<'a>) -> Self {
        Self::exact(VersionOperation::Dependencies, name, version)
    }
    /// Deprecated upstream compatibility operation; returns no author identities.
    pub fn authors(name: CrateName<'a>, version: Version<'a>) -> Self {
        Self::exact(VersionOperation::Authors, name, version)
    }
    /// Requests the JSON README URL profile, never a redirect-following HTML request.
    pub fn readme(name: CrateName<'a>, version: Version<'a>) -> Self {
        Self::exact(VersionOperation::Readme, name, version)
    }
    /// Selected immutable operation.
    pub const fn operation(self) -> VersionOperation {
        self.operation
    }
    /// Validated list query, absent for exact-version routes.
    pub const fn query(self) -> Option<Query<'a>> {
        self.query
    }
    pub(super) fn parts(self) -> ([P<'a>; 4], usize) {
        let last = match self.operation {
            VersionOperation::Dependencies => F::Dependencies,
            VersionOperation::Authors => F::Authors,
            VersionOperation::Readme => F::Readme,
            _ => F::Versions,
        };
        (
            [
                P::Fixed(F::Crates),
                P::Crate(self.name),
                self.version
                    .map(P::Version)
                    .unwrap_or(P::Fixed(F::Versions)),
                P::Fixed(last),
            ],
            if matches!(
                self.operation,
                VersionOperation::List | VersionOperation::Detail
            ) {
                3
            } else {
                4
            },
        )
    }
    /// Atomic bounded target encoding; capacity failures leave output untouched.
    pub fn write_target(self, output: &mut [u8]) -> Result<ApiRequestTarget<'_>, QueryError> {
        let (parts, len) = self.parts();
        let path = ApiPath::new(parts.get(..len).ok_or(QueryError::Output)?)?;
        match self.query {
            Some(query) => query.write_target(path, output),
            None => path.write(output),
        }
    }
}
