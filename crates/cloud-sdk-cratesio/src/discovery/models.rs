use super::{DiscoveryError as Error, DiscoveryValue, SummaryCrate};
use crate::query::Page;
use alloc::{string::String, vec::Vec};

macro_rules! model {
    ($(#[$meta:meta])* $name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        $(#[$meta])* pub struct $name { $(#[doc = stringify!($field)] pub $field: $ty,)* }
        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(concat!(stringify!($name), "([redacted])"))
            }
        }
    };
}
pub(super) use model;
model!(/// A category including optional, bounded parent and child collections.
    Category { id: String, category: String, slug: String, description: String,
        created_at: Timestamp, crates_cnt: u32, parent_categories: Option<Vec<Category>>,
        subcategories: Option<Vec<Category>> });
model!(/// One keyword record. IDs are opaque response text, not numeric IDs.
    Keyword { id: String, keyword: String, created_at: Timestamp, crates_cnt: u32 });
model!(/// Complete slug catalog entry.
    CategorySlug { id: String, slug: String, description: String });
model!(/// Public deployment information. CDN/banner values are untrusted data:
    /// never use them to choose credential destinations or render raw HTML.
    SiteMetadata { deployed_sha: String, commit: String, cdn_base: String,
        read_only: bool, banner_message: Option<String> });
model!(/// Complete source-owned front-page summary, including all crate records.
    Summary { num_downloads: u64, num_crates: u64, new_crates: Vec<SummaryCrate>,
        most_downloaded: Vec<SummaryCrate>, most_recently_downloaded: Vec<SummaryCrate>,
        just_updated: Vec<SummaryCrate>, popular_keywords: Vec<Keyword>,
        popular_categories: Vec<Category> });

/// Checked RFC 3339 calendar timestamp, retaining source spelling.
/// The SDK accepts up to nine fractional digits and rejects leap seconds.
pub struct Timestamp(String);
impl core::fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Timestamp([redacted])")
    }
}
impl Timestamp {
    /// Exact provider timestamp; no implicit timezone conversion.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub(super) fn parse(value: &DiscoveryValue) -> Result<Self, Error> {
        let value = value.text(35)?;
        let (local, suffix) = if value.ends_with(['Z', 'z']) {
            (
                value
                    .get(..value.len().checked_sub(1).ok_or(Error::Value)?)
                    .ok_or(Error::Value)?,
                "Z",
            )
        } else {
            let at = value.len().checked_sub(6).ok_or(Error::Value)?;
            (
                value.get(..at).ok_or(Error::Value)?,
                value.get(at..).ok_or(Error::Value)?,
            )
        };
        if suffix != "Z" {
            let bytes = suffix.as_bytes();
            if !matches!(bytes.first(), Some(b'+' | b'-'))
                || bytes.get(3) != Some(&b':')
                || ![1, 2, 4, 5]
                    .iter()
                    .all(|i| bytes.get(*i).is_some_and(u8::is_ascii_digit))
                || suffix
                    .get(1..3)
                    .and_then(|v| v.parse::<u8>().ok())
                    .is_none_or(|v| v > 23)
                || suffix
                    .get(4..6)
                    .and_then(|v| v.parse::<u8>().ok())
                    .is_none_or(|v| v > 59)
            {
                return Err(Error::Value);
            }
        }
        if local.len() > 29 {
            return Err(Error::Value);
        }
        let mut canonical = [0; 30];
        canonical
            .get_mut(..local.len())
            .ok_or(Error::Value)?
            .copy_from_slice(local.as_bytes());
        *canonical.get_mut(10).ok_or(Error::Value)? = match local.as_bytes().get(10) {
            Some(b'T' | b't') => b'T',
            _ => return Err(Error::Value),
        };
        *canonical.get_mut(local.len()).ok_or(Error::Value)? = b'Z';
        let text = core::str::from_utf8(
            canonical
                .get(..local.len().checked_add(1).ok_or(Error::Value)?)
                .ok_or(Error::Value)?,
        )
        .map_err(|_| Error::Value)?;
        cloud_sdk::async_resource::AsyncResourceTimestamp::parse(text).map_err(|_| Error::Value)?;
        Ok(Self(value))
    }
}

/// Numbered continuation without hiding the conservative page-depth ceiling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageContinuation {
    /// The current count indicates there is no next page.
    End,
    /// Another numbered page is within the SDK policy.
    Page(Page),
    /// More data exists, but this SDK's page-depth ceiling was reached.
    LimitReached,
}
/// Bounded decoded list and checked continuation metadata.
#[derive(Debug)]
pub struct DiscoveryPage<T> {
    /// Fully decoded items, no more than the requested page size.
    pub items: Vec<T>,
    /// Source total; may change between requests.
    pub total: u64,
    /// Derived from the requested page and source total, or a validated link.
    pub next: PageContinuation,
    /// Previous page, when representable.
    pub previous: Option<Page>,
}
/// Operation-specific success selected only after status, JSON and model checks.
#[derive(Debug)]
pub enum DiscoveryResponse {
    /// Category list.
    Categories(DiscoveryPage<Category>),
    /// Category detail.
    Category(Category),
    /// Slug catalog.
    CategorySlugs(Vec<CategorySlug>),
    /// Keyword list.
    Keywords(DiscoveryPage<Keyword>),
    /// Keyword detail.
    Keyword(Keyword),
    /// Site metadata.
    SiteMetadata(SiteMetadata),
    /// Front-page summary.
    Summary(Summary),
}

pub(super) fn text(v: &DiscoveryValue, name: &str, max: usize) -> Result<String, Error> {
    v.required(name)?.text(max)
}
pub(super) fn count(v: &DiscoveryValue, name: &str) -> Result<u64, Error> {
    v.required(name)?.count(i64::MAX as u64)
}
pub(super) fn small_count(v: &DiscoveryValue, name: &str) -> Result<u32, Error> {
    u32::try_from(v.required(name)?.count(i32::MAX as u64)?).map_err(|_| Error::Value)
}
pub(super) fn nullable<T>(
    v: &DiscoveryValue,
    parse: impl FnOnce(&DiscoveryValue) -> Result<T, Error>,
) -> Result<Option<T>, Error> {
    if v.is_null() {
        Ok(None)
    } else {
        parse(v).map(Some)
    }
}
pub(super) fn list<T>(
    v: &DiscoveryValue,
    max: usize,
    mut parse: impl FnMut(&DiscoveryValue) -> Result<T, Error>,
) -> Result<Vec<T>, Error> {
    let values = v.array()?;
    if values.len() > max {
        return Err(Error::Limit);
    }
    let mut out = Vec::new();
    out.try_reserve_exact(values.len())
        .map_err(|_| Error::Allocation)?;
    for value in values {
        out.push(parse(value)?);
    }
    Ok(out)
}
pub(super) fn category(v: &DiscoveryValue, depth: usize) -> Result<Category, Error> {
    if depth > 8 {
        return Err(Error::Limit);
    }
    let related = |name| {
        v.get(name)?
            .map(|v| {
                list(v, 1024, |v| {
                    category(v, depth.checked_add(1).ok_or(Error::Limit)?)
                })
            })
            .transpose()
    };
    Ok(Category {
        id: text(v, "id", 256)?,
        category: text(v, "category", 1024)?,
        slug: text(v, "slug", 256)?,
        description: text(v, "description", 65_536)?,
        created_at: Timestamp::parse(v.required("created_at")?)?,
        crates_cnt: small_count(v, "crates_cnt")?,
        parent_categories: related("parent_categories")?,
        subcategories: related("subcategories")?,
    })
}
pub(super) fn keyword(v: &DiscoveryValue) -> Result<Keyword, Error> {
    Ok(Keyword {
        id: text(v, "id", 256)?,
        keyword: text(v, "keyword", 256)?,
        created_at: Timestamp::parse(v.required("created_at")?)?,
        crates_cnt: small_count(v, "crates_cnt")?,
    })
}
