use super::{
    IncludeSet, MAX_QUERY_PARAMETERS, Page, PerPage, QueryError, QueryOperation, SearchQuery, Seek,
    Sort,
};
use crate::identifiers::{CategorySlug, CrateName, Date, Keyword, NumericId, Version};

/// Typed query parameter groups; array variants own their repeated wire key.
#[derive(Clone, Copy, Debug)]
pub enum Parameter<'a> {
    /// Search text (`q`).
    Search(SearchQuery<'a>),
    /// Category hierarchy filter.
    Category(CategorySlug<'a>),
    /// One keyword filter.
    Keyword(Keyword<'a>),
    /// Space-separated intersection of keywords.
    AllKeywords(&'a [Keyword<'a>]),
    /// One ASCII crate-name starting letter.
    Letter(char),
    /// Individual owner filter.
    UserId(NumericId),
    /// Team owner filter.
    TeamId(NumericId),
    /// Explicit followed-crates filter, encoded as `yes`.
    Following,
    /// Exact crate names, using repeated `ids[]` pairs.
    Ids(&'a [CrateName<'a>]),
    /// Exact versions, using repeated `nums[]` pairs.
    Versions(&'a [Version<'a>]),
    /// Include yanked crates, encoded as `yes`.
    IncludeYanked,
    /// Trusted-publishing crate filter.
    Crate(CrateName<'a>),
    /// Additional response sections.
    Include(IncludeSet<'a>),
    /// Source-owned sort order.
    Sort(Sort),
    /// Numbered continuation, mutually exclusive with seek.
    Page(Page),
    /// Explicit page size.
    PerPage(PerPage),
    /// Opaque provider continuation, mutually exclusive with page.
    Seek(Seek<'a>),
    /// Download-count date filter.
    BeforeDate(Date<'a>),
}
impl Parameter<'_> {
    pub(super) const fn key(self) -> &'static str {
        match self {
            Self::Search(_) => "q",
            Self::Category(_) => "category",
            Self::Keyword(_) => "keyword",
            Self::AllKeywords(_) => "all_keywords",
            Self::Letter(_) => "letter",
            Self::UserId(_) => "user_id",
            Self::TeamId(_) => "team_id",
            Self::Following => "following",
            Self::Ids(_) => "ids[]",
            Self::Versions(_) => "nums[]",
            Self::IncludeYanked => "include_yanked",
            Self::Crate(_) => "crate",
            Self::Include(_) => "include",
            Self::Sort(_) => "sort",
            Self::Page(_) => "page",
            Self::PerPage(_) => "per_page",
            Self::Seek(_) => "seek",
            Self::BeforeDate(_) => "before_date",
        }
    }
    fn exclusive_filter(self) -> bool {
        matches!(
            self,
            Self::AllKeywords(_)
                | Self::Keyword(_)
                | Self::Letter(_)
                | Self::UserId(_)
                | Self::TeamId(_)
                | Self::Following
                | Self::Ids(_)
        )
    }
}

/// Immutable operation-scoped query snapshot, with deterministic parameter order.
#[derive(Clone, Copy, Debug)]
pub struct Query<'a> {
    pub(super) operation: QueryOperation,
    pub(super) parameters: &'a [Parameter<'a>],
}
impl<'a> Query<'a> {
    /// Validates every group before any output is touched.
    pub fn new(
        operation: QueryOperation,
        parameters: &'a [Parameter<'a>],
    ) -> Result<Self, QueryError> {
        if parameters.len() > MAX_QUERY_PARAMETERS {
            return Err(QueryError::Limit);
        }
        for (i, parameter) in parameters.iter().enumerate() {
            let previous = parameters.get(..i).ok_or(QueryError::Limit)?;
            if previous.iter().any(|p| p.key() == parameter.key()) {
                return Err(QueryError::Duplicate);
            }
            if previous.iter().any(|p| {
                matches!(
                    (p, parameter),
                    (Parameter::Page(_), Parameter::Seek(_))
                        | (Parameter::Seek(_), Parameter::Page(_))
                )
            }) {
                return Err(QueryError::Conflict);
            }
            if operation == QueryOperation::Crates
                && parameter.exclusive_filter()
                && previous.iter().any(|p| p.exclusive_filter())
            {
                return Err(QueryError::Conflict);
            }
            validate(operation, *parameter)?;
        }
        Ok(Self {
            operation,
            parameters,
        })
    }
    /// Returns the exact operation this snapshot was checked against.
    #[must_use]
    pub const fn operation(self) -> QueryOperation {
        self.operation
    }
    /// Returns a numbered starting page, or none for seek/unset pagination.
    #[must_use]
    pub fn page(self) -> Option<Page> {
        self.parameters.iter().find_map(|p| match p {
            Parameter::Page(p) => Some(*p),
            _ => None,
        })
    }
    /// Returns the unchanged opaque seek token, if present.
    #[must_use]
    pub fn seek(self) -> Option<Seek<'a>> {
        self.parameters.iter().find_map(|p| match p {
            Parameter::Seek(s) => Some(*s),
            _ => None,
        })
    }
    /// Returns the requested page size or the source's default of ten entries.
    #[must_use]
    pub fn per_page(self) -> u32 {
        self.parameters
            .iter()
            .find_map(|p| match p {
                Parameter::PerPage(p) => Some(p.get()),
                _ => None,
            })
            .unwrap_or(10)
    }
}

fn unique<T: PartialEq>(values: &[T]) -> Result<(), QueryError> {
    if values.is_empty() || values.len() > MAX_QUERY_PARAMETERS {
        return Err(QueryError::Limit);
    }
    for (i, value) in values.iter().enumerate() {
        if values.get(..i).ok_or(QueryError::Limit)?.contains(value) {
            return Err(QueryError::Duplicate);
        }
    }
    Ok(())
}

fn validate(op: QueryOperation, p: Parameter<'_>) -> Result<(), QueryError> {
    use Parameter as P;
    use QueryOperation as O;
    let allowed = match p {
        P::Page(_) | P::PerPage(_) | P::Seek(_) => op.paginated(),
        P::Sort(sort) => op.sort(sort),
        P::Include(set) => set.values().iter().all(|v| op.include(*v)),
        P::UserId(_) => matches!(op, O::Crates | O::GithubConfigs | O::GitlabConfigs),
        P::Crate(_) => matches!(op, O::GithubConfigs | O::GitlabConfigs),
        P::BeforeDate(_) => op == O::VersionDownloads,
        P::Versions(values) => {
            unique(values)?;
            op == O::Versions
        }
        P::Ids(values) => {
            unique(values)?;
            op == O::Crates
        }
        P::AllKeywords(values) => {
            unique(values)?;
            op == O::Crates
        }
        P::Letter(letter) => {
            if !letter.is_ascii_alphabetic() {
                return Err(QueryError::Syntax);
            }
            op == O::Crates
        }
        P::Search(_)
        | P::Category(_)
        | P::Keyword(_)
        | P::TeamId(_)
        | P::Following
        | P::IncludeYanked => op == O::Crates,
    };
    if allowed {
        Ok(())
    } else {
        Err(QueryError::Operation)
    }
}
