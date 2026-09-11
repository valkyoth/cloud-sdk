use super::CatalogContinuation;
pub use crate::discovery::CrateLinks;
pub use crate::discovery::SummaryCrate as CrateRecord;
use crate::discovery::models::model;
use crate::discovery::{Category, DiscoveryValue, Keyword};
use alloc::{string::String, vec::Vec};

model!(/// Crates.io list profile; the richer record is not confused with Cargo's minimum response.
    CratePage { items: Vec<CrateRecord>, total: u64, next: CatalogContinuation, previous: Option<super::CatalogLink> });
model!(/// Stable Cargo search entry; absent nullable descriptions remain absent.
    CargoSearchCrate { name: String, max_version: String, description: Option<String> });
model!(/// Stable Cargo search result. Total does not authorize bulk traversal.
    CargoSearchPage { items: Vec<CargoSearchCrate>, total: u64 });
model!(/// Complete metadata, with explicitly nullable include expansions.
    CrateMetadata { krate: CrateRecord, versions: Option<Vec<IncludedVersion>>,
        keywords: Option<Vec<Keyword>>, categories: Option<Vec<Category>> });

/// Bounded, schema-checked included version. The full field tree is retained,
/// including future fields. Version-specific endpoint operations arrive separately.
/// All links, URLs, feature and publisher data are inert untrusted metadata.
#[derive(Debug)]
pub struct IncludedVersion(pub(super) DiscoveryValue);
impl IncludedVersion {
    /// Inspect the complete protected record without a raw JSON copy.
    pub const fn fields(&self) -> &DiscoveryValue {
        &self.0
    }
}

/// Success selected by the immutable request profile.
#[derive(Debug)]
// Keep bounded success records inline instead of adding an infallible Box allocation.
#[allow(clippy::large_enum_variant)]
pub enum CatalogResponse {
    /// Full crates.io list.
    Crates(CratePage),
    /// Stable Cargo search.
    CargoSearch(CargoSearchPage),
    /// Metadata for a named or literal-new crate.
    Crate(CrateMetadata),
}
