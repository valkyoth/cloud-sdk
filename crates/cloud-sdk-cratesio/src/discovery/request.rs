use crate::{
    endpoint::ApiRequestTarget,
    identifiers::{CategorySlug, Keyword},
    query::{
        ApiPath, FixedSegment as F, Parameter, PathSegment as P, Query, QueryError, QueryOperation,
    },
};
use cloud_sdk::operation::{
    CostIntent, OperationId, OperationImpact, OperationMetadata, OperationMetadataError,
    RequestIdPolicy, RequestSemantics, RetryEligibility,
};

/// Source-locked, anonymous GET discovery operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryOperation {
    /// List top-level categories.
    Categories,
    /// Inspect a category, its parents and children.
    Category,
    /// List every category slug.
    CategorySlugs,
    /// List keywords.
    Keywords,
    /// Inspect one keyword.
    Keyword,
    /// Inspect public site deployment metadata.
    SiteMetadata,
    /// Inspect front-page statistics and collections.
    Summary,
}
impl DiscoveryOperation {
    /// Exact source operation identifier.
    #[must_use]
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::Categories => "list_categories",
            Self::Category => "find_category",
            Self::CategorySlugs => "list_category_slugs",
            Self::Keywords => "list_keywords",
            Self::Keyword => "find_keyword",
            Self::SiteMetadata => "get_site_metadata",
            Self::Summary => "get_summary",
        }
    }
    /// Typed operation identity; construction checks the fixed source constant.
    pub fn operation_id(self) -> Result<OperationId, cloud_sdk::operation::OperationIdError> {
        OperationId::new(self.operation_name())
    }
    /// Read-only, safe, no known cost; these entry points never retry.
    pub const fn metadata(self) -> Result<OperationMetadata, OperationMetadataError> {
        OperationMetadata::new(
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::Never,
            CostIntent::NoKnownCost,
            RequestIdPolicy::Discard,
        )
    }
}

/// Immutable path/query snapshot. No raw path, URL or header escape hatch.
#[derive(Clone, Copy, Debug)]
pub struct DiscoveryRequest<'a> {
    operation: DiscoveryOperation,
    category: Option<CategorySlug<'a>>,
    keyword: Option<Keyword<'a>>,
    query: Option<Query<'a>>,
}
impl<'a> DiscoveryRequest<'a> {
    const fn empty(operation: DiscoveryOperation) -> Self {
        Self {
            operation,
            category: None,
            keyword: None,
            query: None,
        }
    }
    /// Category list with source-supported numbered pagination and sorting.
    pub fn categories(parameters: &'a [Parameter<'a>]) -> Result<Self, QueryError> {
        Self::list(
            DiscoveryOperation::Categories,
            QueryOperation::Categories,
            parameters,
        )
    }
    /// Keyword list with source-supported numbered pagination and sorting.
    pub fn keywords(parameters: &'a [Parameter<'a>]) -> Result<Self, QueryError> {
        Self::list(
            DiscoveryOperation::Keywords,
            QueryOperation::Keywords,
            parameters,
        )
    }
    fn list(
        operation: DiscoveryOperation,
        query: QueryOperation,
        parameters: &'a [Parameter<'a>],
    ) -> Result<Self, QueryError> {
        let query = Query::new(query, parameters)?;
        // OpenAPI advertises the shared seek parameter, but these pinned
        // controllers use offset pagination and reject seek at execution.
        if query.seek().is_some() {
            return Err(QueryError::Operation);
        }
        Ok(Self {
            query: Some(query),
            ..Self::empty(operation)
        })
    }
    /// Category details, with a validated hierarchical slug.
    #[must_use]
    pub const fn category(category: CategorySlug<'a>) -> Self {
        Self {
            category: Some(category),
            ..Self::empty(DiscoveryOperation::Category)
        }
    }
    /// Keyword details, with a validated keyword.
    #[must_use]
    pub const fn keyword(keyword: Keyword<'a>) -> Self {
        Self {
            keyword: Some(keyword),
            ..Self::empty(DiscoveryOperation::Keyword)
        }
    }
    /// Complete slug catalog.
    #[must_use]
    pub const fn category_slugs() -> Self {
        Self::empty(DiscoveryOperation::CategorySlugs)
    }
    /// Public deployment information; returned CDN text is never a destination.
    #[must_use]
    pub const fn site_metadata() -> Self {
        Self::empty(DiscoveryOperation::SiteMetadata)
    }
    /// Front-page aggregate data.
    #[must_use]
    pub const fn summary() -> Self {
        Self::empty(DiscoveryOperation::Summary)
    }
    /// Exact request operation.
    #[must_use]
    pub const fn operation(self) -> DiscoveryOperation {
        self.operation
    }
    /// Original list query, if this is a paginated operation.
    #[must_use]
    pub const fn query(self) -> Option<Query<'a>> {
        self.query
    }
    pub(super) fn segments(self) -> ([P<'a>; 2], usize) {
        let root = match self.operation {
            DiscoveryOperation::Categories | DiscoveryOperation::Category => F::Categories,
            DiscoveryOperation::Keywords | DiscoveryOperation::Keyword => F::Keywords,
            DiscoveryOperation::CategorySlugs => F::CategorySlugs,
            DiscoveryOperation::SiteMetadata => F::SiteMetadata,
            DiscoveryOperation::Summary => F::Summary,
        };
        let child = self
            .category
            .map(P::Category)
            .or_else(|| self.keyword.map(P::Keyword));
        (
            [P::Fixed(root), child.unwrap_or(P::Fixed(root))],
            if child.is_some() { 2 } else { 1 },
        )
    }
    /// Atomically writes the complete encoded origin-form target.
    pub fn write_target(self, output: &mut [u8]) -> Result<ApiRequestTarget<'_>, QueryError> {
        let (segments, len) = self.segments();
        let path = ApiPath::new(segments.get(..len).ok_or(QueryError::Output)?)?;
        match self.query {
            Some(query) => query.write_target(path, output),
            None => path.write(output),
        }
    }
}
