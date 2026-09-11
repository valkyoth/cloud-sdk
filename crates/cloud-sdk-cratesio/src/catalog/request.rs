use crate::{
    endpoint::ApiRequestTarget,
    identifiers::CrateName,
    query::{
        ApiPath, FixedSegment as F, Parameter, PathSegment as P, Query, QueryError, QueryOperation,
    },
};
use cloud_sdk::operation::{
    OperationId, OperationIdError, OperationMetadata, OperationMetadataError,
};

/// Reviewed search and metadata GET operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogOperation {
    /// Full crates.io search response.
    List,
    /// Stable Cargo search response profile on the same route.
    CargoSearch,
    /// Named crate metadata.
    Crate,
    /// Literal GET /crates/new, never a publication request.
    NewCrate,
}
impl CatalogOperation {
    /// Source operation identifier.
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::List | Self::CargoSearch => "list_crates",
            Self::Crate => "find_crate",
            Self::NewCrate => "find_new_crate",
        }
    }
    /// Typed source identifier.
    pub fn operation_id(self) -> Result<OperationId, OperationIdError> {
        OperationId::new(self.operation_name())
    }
    /// Read-only GET, with no automatic retry or known cost.
    pub const fn metadata(self) -> Result<OperationMetadata, OperationMetadataError> {
        crate::discovery::DiscoveryOperation::Summary.metadata()
    }
}

/// Immutable checked query and path; no raw route or credential escape hatch.
#[derive(Clone, Copy, Debug)]
pub struct CatalogRequest<'a> {
    pub(super) operation: CatalogOperation,
    pub(super) name: Option<CrateName<'a>>,
    pub(super) query: Query<'a>,
}
impl<'a> CatalogRequest<'a> {
    /// Full website search/list, supporting all reviewed query selectors.
    pub fn list(parameters: &'a [Parameter<'a>]) -> Result<Self, QueryError> {
        Ok(Self {
            operation: CatalogOperation::List,
            name: None,
            query: Query::new(QueryOperation::Crates, parameters)?,
        })
    }
    /// Stable Cargo search profile. Only q/per_page are admitted, no tokens.
    /// Omit q for an empty search; an explicitly empty SearchQuery is rejected.
    pub fn cargo_search(parameters: &'a [Parameter<'a>]) -> Result<Self, QueryError> {
        if parameters
            .iter()
            .any(|p| !matches!(p, Parameter::Search(_) | Parameter::PerPage(_)))
        {
            return Err(QueryError::Operation);
        }
        Ok(Self {
            operation: CatalogOperation::CargoSearch,
            ..Self::list(parameters)?
        })
    }
    /// Metadata with explicit include selection (omission uses upstream full).
    /// The reserved literal `new` selects its distinct source operation.
    pub fn crate_metadata(
        name: CrateName<'a>,
        parameters: &'a [Parameter<'a>],
    ) -> Result<Self, QueryError> {
        Ok(Self {
            operation: if name.as_str() == "new" {
                CatalogOperation::NewCrate
            } else {
                CatalogOperation::Crate
            },
            name: Some(name),
            query: Query::new(QueryOperation::Crate, parameters)?,
        })
    }
    /// Explicit literal-new lookup with the same source implementation includes.
    pub fn new_crate(parameters: &'a [Parameter<'a>]) -> Result<Self, QueryError> {
        Self::crate_metadata(
            CrateName::new("new").map_err(|_| QueryError::Syntax)?,
            parameters,
        )
    }
    /// Exact operation profile.
    pub const fn operation(self) -> CatalogOperation {
        self.operation
    }
    /// Immutable validated parameters.
    pub const fn query(self) -> Query<'a> {
        self.query
    }
    pub(super) fn segments(self) -> ([P<'a>; 2], usize) {
        (
            [
                P::Fixed(F::Crates),
                self.name.map(P::Crate).unwrap_or(P::Fixed(F::Crates)),
            ],
            if self.name.is_some() { 2 } else { 1 },
        )
    }
    /// Atomically encodes a bounded complete target, preserving output on error.
    pub fn write_target(self, output: &mut [u8]) -> Result<ApiRequestTarget<'_>, QueryError> {
        let (parts, len) = self.segments();
        self.query.write_target(
            ApiPath::new(parts.get(..len).ok_or(QueryError::Output)?)?,
            output,
        )
    }
    #[cfg(any(feature = "blocking", feature = "async"))]
    pub(crate) fn following(self) -> bool {
        self.query
            .parameters()
            .iter()
            .any(|p| matches!(p, Parameter::Following))
    }
}
