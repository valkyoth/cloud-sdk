use super::{CatalogError as Error, CatalogRequest};
use crate::{
    discovery::{DiscoveryValue as Value, models::count},
    endpoint::OfficialCratesIoEndpoint,
    pagination::{Direction, PageLink},
    query::{ApiPath, Page, Parameter, Sort},
};
use alloc::string::String;

/// Complete checked continuation query. Debug never discloses query text.
pub struct CatalogLink(String);
impl core::fmt::Debug for CatalogLink {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("CatalogLink([redacted])")
    }
}
impl CatalogLink {
    /// Revalidates against the exact original request before explicit use.
    pub fn validate<'a>(
        &'a self,
        endpoint: OfficialCratesIoEndpoint,
        path: ApiPath<'a>,
        query: crate::query::Query<'a>,
        direction: Direction,
    ) -> Result<PageLink<'a>, crate::pagination::PaginationError> {
        PageLink::new(endpoint, path, query, &self.0, direction)
    }
    /// Inert validated link text; never an authorization capability.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
/// Continuation never confuses known truncation with completion.
#[derive(Debug)]
pub enum CatalogContinuation {
    /// Provider supplied no next link; totals are not snapshot isolation.
    End,
    /// Next link, requiring an explicit request and traversal budget.
    Next(CatalogLink),
    /// Relevance or SDK numbered-page ceiling reached with more matches.
    LimitReached,
}

pub(super) fn links(
    request: CatalogRequest<'_>,
    endpoint: OfficialCratesIoEndpoint,
    meta: &Value,
) -> Result<(u64, CatalogContinuation, Option<CatalogLink>), Error> {
    let total = count(meta, "total")?;
    let next = meta.required("next_page")?;
    let previous = meta.required("prev_page")?;
    let (parts, len) = request.segments();
    let path = ApiPath::new(parts.get(..len).ok_or(Error::Binding)?).map_err(|_| Error::Binding)?;
    let query = request.query;
    let has_search = query
        .parameters()
        .iter()
        .any(|p| matches!(p, Parameter::Search(_)));
    let relevance = has_search
        && query
            .parameters()
            .iter()
            .find_map(|p| match p {
                Parameter::Sort(s) => Some(*s == Sort::Relevance),
                _ => None,
            })
            .unwrap_or(has_search);
    let reachable = if relevance { total.min(1000) } else { total };
    let current = query.page().map(Page::get).unwrap_or(1);
    let end = u64::from(current)
        .checked_mul(u64::from(query.per_page()))
        .ok_or(Error::Limit)?;
    let numbered = query.seek().is_none();
    let ceiling = query
        .page()
        .is_some_and(|p| p.get() == crate::query::MAX_PAGE)
        && end < total;
    let next = if next.is_null() {
        if ceiling || (relevance && total > 1000 && (query.seek().is_some() || end >= 1000)) {
            CatalogContinuation::LimitReached
        } else if numbered && end < reachable {
            return Err(Error::Binding);
        } else {
            CatalogContinuation::End
        }
    } else {
        let text = next.text(crate::query::MAX_TARGET_BYTES)?;
        if numbered && end >= total {
            return Err(Error::Binding);
        }
        if PageLink::next_at_limit(endpoint, path, query, &text).map_err(|_| Error::Binding)? {
            CatalogContinuation::LimitReached
        } else {
            PageLink::new(endpoint, path, query, &text, Direction::Next)
                .map_err(|_| Error::Binding)?;
            CatalogContinuation::Next(CatalogLink(text))
        }
    };
    let previous = if previous.is_null() {
        if query.page().is_some_and(|p| p.get() > 1) {
            return Err(Error::Binding);
        }
        None
    } else {
        if query.page().is_none() {
            return Err(Error::Binding);
        }
        let text = previous.text(crate::query::MAX_TARGET_BYTES)?;
        PageLink::new(endpoint, path, query, &text, Direction::Previous)
            .map_err(|_| Error::Binding)?;
        Some(CatalogLink(text))
    };
    Ok((total, next, previous))
}
