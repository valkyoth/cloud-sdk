use super::{
    CategorySlug, DiscoveryError as Error, DiscoveryOperation as Op, DiscoveryPage,
    DiscoveryRequest, DiscoveryResponse as Response, DiscoveryValue as Value, PageContinuation,
    SiteMetadata, Summary, crate_model::crate_record, models::*, value::Builder,
};
use crate::{
    endpoint::{OfficialCratesIoEndpoint, OfficialEndpointPurpose},
    pagination::{Cursor, Direction, PageLink},
    query::{ApiPath, Page},
    wire::JsonSuccess,
};
use alloc::vec::Vec;
use cloud_sdk::incremental_json::IncrementalJsonProgress;

impl DiscoveryRequest<'_> {
    /// Decodes only a previously admitted JSON success, consuming and clearing
    /// its response storage. All unknown fields were validated by the same
    /// incremental parser and remain subject to depth, node and text bounds.
    pub fn decode(
        self,
        endpoint: OfficialCratesIoEndpoint,
        success: JsonSuccess<'_>,
    ) -> Result<Response, Error> {
        if endpoint.purpose() == OfficialEndpointPurpose::StaticDownloads {
            return Err(Error::Binding);
        }
        let mut builder = Builder::default();
        let progress = success
            .visit(&mut builder)
            .map_err(|e| e.into_visitor_error().unwrap_or(Error::Json))?;
        if progress != IncrementalJsonProgress::Complete {
            return Err(Error::Json);
        }
        self.model(endpoint, builder.finish()?)
    }
    fn model(self, endpoint: OfficialCratesIoEndpoint, mut root: Value) -> Result<Response, Error> {
        match self.operation() {
            Op::Categories => self
                .page(endpoint, &root, "categories", |v| category(v, 0))
                .map(Response::Categories),
            Op::Keywords => self
                .page(endpoint, &root, "keywords", keyword)
                .map(Response::Keywords),
            Op::Category => {
                let item = category(root.required("category")?, 0)?;
                let (segments, _) = self.segments();
                if !matches!(segments.get(1), Some(crate::query::PathSegment::Category(slug)) if slug.as_str() == item.slug)
                {
                    return Err(Error::Binding);
                }
                Ok(Response::Category(item))
            }
            Op::Keyword => {
                let item = keyword(root.required("keyword")?)?;
                let (segments, _) = self.segments();
                if !matches!(segments.get(1), Some(crate::query::PathSegment::Keyword(name)) if name.as_str().eq_ignore_ascii_case(&item.keyword))
                {
                    return Err(Error::Binding);
                }
                Ok(Response::Keyword(item))
            }
            Op::CategorySlugs => list(root.required("category_slugs")?, 1024, |v| {
                Ok(CategorySlug {
                    id: text(v, "id", 256)?,
                    slug: text(v, "slug", 256)?,
                    description: text(v, "description", 65_536)?,
                })
            })
            .map(Response::CategorySlugs),
            Op::SiteMetadata => Ok(Response::SiteMetadata(SiteMetadata {
                deployed_sha: text(&root, "deployed_sha", 256)?,
                commit: text(&root, "commit", 256)?,
                cdn_base: text(&root, "cdn_base", 4096)?,
                read_only: root.required("read_only")?.boolean()?,
                banner_message: root
                    .get("banner_message")?
                    .map(|v| v.text(65_536))
                    .transpose()?,
            })),
            Op::Summary => {
                let num_downloads = count(&root, "num_downloads")?;
                let num_crates = count(&root, "num_crates")?;
                let popular_keywords = list(root.required("popular_keywords")?, 10, keyword)?;
                let popular_categories =
                    list(root.required("popular_categories")?, 10, |v| category(v, 0))?;
                let mut records = |name| {
                    let values = root.take(name)?.into_array()?;
                    if values.len() > 10 {
                        return Err(Error::Limit);
                    }
                    let mut out = Vec::new();
                    out.try_reserve_exact(values.len())
                        .map_err(|_| Error::Allocation)?;
                    for mut value in values {
                        out.push(crate_record(&mut value)?);
                    }
                    Ok(out)
                };
                Ok(Response::Summary(Summary {
                    num_downloads,
                    num_crates,
                    popular_keywords,
                    popular_categories,
                    new_crates: records("new_crates")?,
                    most_downloaded: records("most_downloaded")?,
                    most_recently_downloaded: records("most_recently_downloaded")?,
                    just_updated: records("just_updated")?,
                }))
            }
        }
    }
    fn page<T>(
        self,
        endpoint: OfficialCratesIoEndpoint,
        root: &Value,
        name: &str,
        parse: impl FnMut(&Value) -> Result<T, Error>,
    ) -> Result<DiscoveryPage<T>, Error> {
        let query = self.query().ok_or(Error::Binding)?;
        let items = list(root.required(name)?, query.per_page() as usize, parse)?;
        let meta = root.required("meta")?;
        let total = count(meta, "total")?;
        let current = query.page().map(Page::get).unwrap_or(1);
        let end = u64::from(current)
            .checked_mul(u64::from(query.per_page()))
            .ok_or(Error::Limit)?;
        let mut next = if end >= total {
            PageContinuation::End
        } else {
            Page::new(current.checked_add(1).ok_or(Error::Limit)?)
                .map(PageContinuation::Page)
                .unwrap_or(PageContinuation::LimitReached)
        };
        let mut previous = current.checked_sub(1).and_then(|p| Page::new(p).ok());
        let (segments, len) = self.segments();
        let path =
            ApiPath::new(segments.get(..len).ok_or(Error::Binding)?).map_err(|_| Error::Binding)?;
        for (key, direction) in [
            ("next_page", Direction::Next),
            ("prev_page", Direction::Previous),
        ] {
            if let Some(value) = meta.get(key)? {
                let page = if value.is_null() {
                    None
                } else {
                    Some(value.with_text(|text| {
                        let link = PageLink::new(endpoint, path, query, text, direction)
                            .map_err(|_| Error::Binding)?;
                        match link.cursor() {
                            Cursor::Page(page) => Ok(page),
                            Cursor::Seek(_) => Err(Error::Binding),
                        }
                    })??)
                };
                match direction {
                    Direction::Next => {
                        next = page
                            .map(PageContinuation::Page)
                            .unwrap_or(PageContinuation::End)
                    }
                    Direction::Previous => previous = page,
                }
            }
        }
        Ok(DiscoveryPage {
            items,
            total,
            next,
            previous,
        })
    }
}
