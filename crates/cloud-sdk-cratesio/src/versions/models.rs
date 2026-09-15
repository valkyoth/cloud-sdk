use super::VersionError;
use crate::discovery::DiscoveryValue;
use alloc::vec::Vec;

/// Schema-checked version metadata, retaining nullable, archived and future fields.
/// License, links, publisher details and feature values are untrusted inert data.
#[derive(Debug)]
pub struct VersionRecord(pub(super) DiscoveryValue);
impl VersionRecord {
    /// Complete protected record; missing and explicit null remain distinguishable.
    pub const fn fields(&self) -> &DiscoveryValue {
        &self.0
    }
}
/// Version-list continuation is explicit; it never authorizes automatic crawling.
#[derive(Debug)]
pub enum VersionContinuation {
    /// No further provider link.
    End,
    /// Inert query link already checked against the original request.
    Next(crate::catalog::CatalogLink),
}
/// Bounded version list with the complete source metadata retained.
#[derive(Debug)]
pub struct VersionPage {
    /// Records bound to the requested crate and optional exact-version filter.
    pub versions: Vec<VersionRecord>,
    /// Source metadata, including optional release tracks.
    pub meta: DiscoveryValue,
    /// Explicit next step; no request is dispatched automatically.
    pub next: VersionContinuation,
}
/// Known kinds are typed; future spellings remain available in the record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DependencyKind {
    /// Normal dependency.
    Normal,
    /// Development dependency.
    Dev,
    /// Build dependency.
    Build,
    /// Unknown source spelling retained, not interpreted as normal.
    Unknown,
}
/// Bounded dependency metadata. Requirement strings are preserved, not resolved.
#[derive(Debug)]
pub struct DependencyRecord(pub(super) DiscoveryValue);
impl DependencyRecord {
    /// Complete protected record including exact requirement, features and target.
    pub const fn fields(&self) -> &DiscoveryValue {
        &self.0
    }
    /// Kind classification never coerces a future value to a known variant.
    pub fn kind(&self) -> Result<DependencyKind, VersionError> {
        self.0.required("kind")?.with_text(|s| match s {
            "normal" => DependencyKind::Normal,
            "dev" => DependencyKind::Dev,
            "build" => DependencyKind::Build,
            _ => DependencyKind::Unknown,
        })
    }
}
/// README URL metadata, never trusted/rendered HTML or a fetch capability.
#[derive(Debug)]
pub struct ReadmeLocation(pub(super) DiscoveryValue);
impl ReadmeLocation {
    /// Inspect the URL text without fetching or forwarding credentials to it.
    pub fn with_url<R>(&self, inspect: impl FnOnce(&str) -> R) -> Result<R, VersionError> {
        self.0.required("url")?.with_text(inspect)
    }
    /// Retains the complete bounded source object, including future fields.
    pub const fn fields(&self) -> &DiscoveryValue {
        &self.0
    }
}
/// Success selected by the exact immutable operation.
#[derive(Debug)]
pub enum VersionResponse {
    /// Version list with source metadata and bounded continuation.
    Versions(VersionPage),
    /// Exact version detail.
    Version(VersionRecord),
    /// Dependency records; repeated crate names across kinds/targets are legal.
    Dependencies(Vec<DependencyRecord>),
    /// Deprecated upstream authors response is empty, not missing data.
    Authors,
    /// JSON location profile; static HTML fetching/rendering remains caller-owned.
    Readme(ReadmeLocation),
}
