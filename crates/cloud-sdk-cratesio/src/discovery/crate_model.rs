use super::{DiscoveryError as Error, DiscoveryValue, Timestamp, models::*};
use alloc::{string::String, vec::Vec};

model!(/// Complete source-owned links. Strings are inert metadata, never followed
    /// or used as credential destinations by discovery execution.
    CrateLinks { version_downloads: String, versions: Option<String>, owners: String,
        owner_team: String, owner_user: String, reverse_dependencies: String });
model!(/// Complete crates.io crate record embedded in the front-page summary.
    /// Nullable fields must be present. Unknown fields are checked then discarded.
    SummaryCrate { id: String, name: String, updated_at: Timestamp,
        versions: Option<Vec<u32>>, keywords: Option<Vec<String>>, categories: Option<Vec<String>>,
        badges: Vec<DiscoveryValue>, created_at: Timestamp, downloads: u64, recent_downloads: Option<u64>,
        default_version: Option<String>, num_versions: u32, yanked: bool, max_version: String,
        newest_version: String, max_stable_version: Option<String>, description: Option<String>,
        homepage: Option<String>, documentation: Option<String>, repository: Option<String>,
        links: CrateLinks, exact_match: bool, trustpub_only: bool });

pub(crate) fn crate_record(v: &mut DiscoveryValue) -> Result<SummaryCrate, Error> {
    let opt_text = |name, max| nullable(v.required(name)?, |v| v.text(max));
    let strings = |name| nullable(v.required(name)?, |v| list(v, 1024, |v| v.text(256)));
    let links = v.required("links")?;
    let links = CrateLinks {
        version_downloads: text(links, "version_downloads", 4096)?,
        versions: nullable(links.required("versions")?, |v| v.text(4096))?,
        owners: text(links, "owners", 4096)?,
        owner_team: text(links, "owner_team", 4096)?,
        owner_user: text(links, "owner_user", 4096)?,
        reverse_dependencies: text(links, "reverse_dependencies", 4096)?,
    };
    let mut out = SummaryCrate {
        id: text(v, "id", 256)?,
        name: text(v, "name", 256)?,
        updated_at: Timestamp::parse(v.required("updated_at")?)?,
        versions: nullable(v.required("versions")?, |v| {
            list(v, 1024, |v| {
                let id = v.count(i32::MAX as u64)?;
                if id == 0 {
                    return Err(Error::Value);
                }
                u32::try_from(id).map_err(|_| Error::Value)
            })
        })?,
        keywords: strings("keywords")?,
        categories: strings("categories")?,
        badges: Vec::new(),
        created_at: Timestamp::parse(v.required("created_at")?)?,
        downloads: count(v, "downloads")?,
        recent_downloads: nullable(v.required("recent_downloads")?, |v| {
            v.count(i64::MAX as u64)
        })?,
        default_version: opt_text("default_version", 150)?,
        num_versions: small_count(v, "num_versions")?,
        yanked: v.required("yanked")?.boolean()?,
        max_version: text(v, "max_version", 150)?,
        newest_version: text(v, "newest_version", 150)?,
        max_stable_version: opt_text("max_stable_version", 150)?,
        description: opt_text("description", 65_536)?,
        homepage: opt_text("homepage", 4096)?,
        documentation: opt_text("documentation", 4096)?,
        repository: opt_text("repository", 4096)?,
        links,
        exact_match: v.required("exact_match")?.boolean()?,
        trustpub_only: v.required("trustpub_only")?.boolean()?,
    };
    out.badges = v.take("badges")?.into_array()?;
    for badge in &out.badges {
        badge.object()?;
    }
    Ok(out)
}
