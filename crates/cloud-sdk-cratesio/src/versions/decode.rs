use super::{VersionError as Error, VersionOperation as Op, *};
use crate::{
    discovery::{DiscoveryValue as Value, value::Builder},
    endpoint::{OfficialCratesIoEndpoint, OfficialEndpointPurpose},
    identifiers::{CrateName, Version},
    pagination::{Direction, PageLink},
    query::{ApiPath, Include, Parameter},
    wire::JsonSuccess,
};
use alloc::vec::Vec;
use cloud_sdk::incremental_json::IncrementalJsonProgress;

impl VersionRequest<'_> {
    /// Decode an admitted JSON success. Storage clears on every exit.
    /// Use VersionClient for dispatch and endpoint/response binding together.
    pub fn decode(
        self,
        endpoint: OfficialCratesIoEndpoint,
        success: JsonSuccess<'_>,
    ) -> Result<VersionResponse, Error> {
        if endpoint.purpose() == OfficialEndpointPurpose::StaticDownloads {
            return Err(Error::Binding);
        }
        let mut builder = Builder::default();
        if success
            .visit(&mut builder)
            .map_err(|e| e.into_visitor_error().unwrap_or(Error::Json))?
            != IncrementalJsonProgress::Complete
        {
            return Err(Error::Json);
        }
        let mut root = builder.finish()?;
        let schema = match self.operation {
            Op::List => Some(schema_table::LIST_VERSIONS),
            Op::Detail => Some(schema_table::FIND_VERSION),
            Op::Dependencies => Some(schema_table::GET_VERSION_DEPENDENCIES),
            Op::Readme => Some(schema_table::GET_VERSION_README),
            Op::Authors => None,
        };
        if let Some(schema) = schema {
            crate::catalog::schema::validate_table(&root, schema, 0, schema_table::NODES)?;
        }
        match self.operation {
            Op::Detail => Ok(VersionResponse::Version(
                self.record(root.take("version")?)?,
            )),
            Op::List => self.page(root, endpoint).map(VersionResponse::Versions),
            Op::Dependencies => {
                dependencies(root.take("dependencies")?).map(VersionResponse::Dependencies)
            }
            Op::Authors => {
                if !root.required("users")?.array()?.is_empty()
                    || !root
                        .required("meta")?
                        .required("names")?
                        .array()?
                        .is_empty()
                {
                    return Err(Error::Binding);
                }
                Ok(VersionResponse::Authors)
            }
            Op::Readme => {
                bounded_text(root.required("url")?, 8192, true)?;
                // A URL is inert data, not a credential destination or a redirect capability.
                Ok(VersionResponse::Readme(ReadmeLocation(root)))
            }
        }
    }
    fn record(self, value: Value) -> Result<VersionRecord, Error> {
        let matches = value
            .required("crate")?
            .with_text(|name| equivalent(self.name.as_str(), name))?;
        if !matches {
            return Err(Error::Binding);
        }
        value
            .required("crate")?
            .with_text(|s| CrateName::new(s).map(|_| ()).map_err(|_| Error::Value))??;
        value.required("num")?.with_text(|s| {
            Version::new(s).map_err(|_| Error::Value)?;
            if self.version.is_some_and(|v| v.as_str() != s) {
                return Err(Error::Binding);
            }
            if let Some(query) = self.query {
                for param in query.parameters() {
                    if let Parameter::Versions(nums) = param
                        && !nums.iter().any(|n| n.as_str() == s)
                    {
                        return Err(Error::Binding);
                    }
                }
            }
            Ok(())
        })??;
        positive_id(value.required("id")?)?;
        if !value
            .required("checksum")?
            .with_text(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))?
        {
            return Err(Error::Value);
        }
        value.required("features")?.visit_fields(|key, values| {
            if key.is_empty() || key.len() > 256 {
                return Err(Error::Limit);
            }
            for value in values.array()? {
                bounded_text(value, 256, true)?;
            }
            Ok(())
        })?;
        Ok(VersionRecord(value))
    }
    fn page(
        self,
        mut root: Value,
        endpoint: OfficialCratesIoEndpoint,
    ) -> Result<VersionPage, Error> {
        let query = self.query.ok_or(Error::Binding)?;
        let values = root.take("versions")?.into_array()?;
        if values.len() > query.per_page() as usize {
            return Err(Error::Limit);
        }
        unique_ids(&values)?;
        let mut versions = Vec::new();
        versions
            .try_reserve_exact(values.len())
            .map_err(|_| Error::Allocation)?;
        for value in values {
            versions.push(self.record(value)?);
        }
        for (i, version) in versions.iter().enumerate() {
            for earlier in versions.get(..i).ok_or(Error::Limit)? {
                if same_text(
                    version.fields().required("num")?,
                    earlier.fields().required("num")?,
                )? {
                    return Err(Error::Binding);
                }
            }
        }
        let meta = root.take("meta")?;
        let total = meta.required("total")?.count(i64::MAX as u64)?;
        if total < versions.len() as u64 || (versions.is_empty() && total != 0) {
            return Err(Error::Binding);
        }
        let include_tracks = query.parameters().iter().any(
            |p| matches!(p, Parameter::Include(s) if s.values().contains(&Include::ReleaseTracks)),
        );
        if meta.get("release_tracks")?.is_some() != include_tracks {
            return Err(Error::Binding);
        }
        if let Some(tracks) = meta.get("release_tracks")? {
            tracks.visit_fields(|key, track| {
                if key.len() > 150 {
                    return Err(Error::Limit);
                }
                track
                    .required("highest")?
                    .with_text(|s| Version::new(s).map(|_| ()).map_err(|_| Error::Value))?
            })?;
        }
        let (parts, len) = self.parts();
        let path =
            ApiPath::new(parts.get(..len).ok_or(Error::Binding)?).map_err(|_| Error::Binding)?;
        let next = meta.required("next_page")?;
        let next = if next.is_null() {
            // The source emits a seek link for every full page, even the last.
            if versions.len() == query.per_page() as usize
                || (query.seek().is_none() && total != versions.len() as u64)
            {
                return Err(Error::Binding);
            }
            VersionContinuation::End
        } else {
            let text = next.text(crate::query::MAX_TARGET_BYTES)?;
            let link = PageLink::new(endpoint, path, query, &text, Direction::Next)
                .map_err(|_| Error::Binding)?;
            if versions.len() != query.per_page() as usize
                || !matches!(link.cursor(), crate::pagination::Cursor::Seek(_))
            {
                return Err(Error::Binding);
            }
            VersionContinuation::Next(crate::catalog::CatalogLink(text))
        };
        Ok(VersionPage {
            versions,
            meta,
            next,
        })
    }
}
fn equivalent(left: &str, right: &str) -> bool {
    let canonical = |b: u8| {
        if b == b'_' {
            b'-'
        } else {
            b.to_ascii_lowercase()
        }
    };
    left.bytes().map(canonical).eq(right.bytes().map(canonical))
}
fn bounded_text(value: &Value, max: usize, nonempty: bool) -> Result<(), Error> {
    value.with_text(|s| {
        if s.len() > max {
            Err(Error::Limit)
        } else if (nonempty && s.is_empty()) || s.chars().any(char::is_control) {
            Err(Error::Value)
        } else {
            Ok(())
        }
    })?
}
fn positive_id(value: &Value) -> Result<u64, Error> {
    let id = value.count(i32::MAX as u64)?;
    if id == 0 { Err(Error::Value) } else { Ok(id) }
}
fn same_text(a: &Value, b: &Value) -> Result<bool, Error> {
    a.with_text(|a| b.with_text(|b| a == b))?
}
fn unique_ids(values: &[Value]) -> Result<(), Error> {
    for (i, value) in values.iter().enumerate() {
        let id = positive_id(value.required("id")?)?;
        for earlier in values.get(..i).ok_or(Error::Limit)? {
            if positive_id(earlier.required("id")?)? == id {
                return Err(Error::Binding);
            }
        }
    }
    Ok(())
}
fn dependencies(value: Value) -> Result<Vec<DependencyRecord>, Error> {
    let values = value.into_array()?;
    unique_ids(&values)?;
    let mut out = Vec::new();
    out.try_reserve_exact(values.len())
        .map_err(|_| Error::Allocation)?;
    let mut version_id = None;
    for value in values {
        let id = positive_id(value.required("version_id")?)?;
        if version_id.is_some_and(|prior| prior != id) {
            return Err(Error::Binding);
        }
        version_id = Some(id);
        value
            .required("crate_id")?
            .with_text(|s| CrateName::new(s).map(|_| ()).map_err(|_| Error::Value))??;
        bounded_text(value.required("req")?, 4096, true)?;
        bounded_text(value.required("kind")?, 64, true)?;
        if !value.required("target")?.is_null() {
            bounded_text(value.required("target")?, 4096, true)?;
        }
        for feature in value.required("features")?.array()? {
            bounded_text(feature, 256, true)?;
        }
        out.push(DependencyRecord(value));
    }
    Ok(out)
}
#[cfg(any(feature = "blocking", feature = "async"))]
impl crate::discovery::checked::CheckedGet for VersionRequest<'_> {
    type Response = VersionResponse;
    fn target(
        self,
        output: &mut [u8],
    ) -> Result<crate::endpoint::ApiRequestTarget<'_>, crate::query::QueryError> {
        self.write_target(output)
    }
    fn decode(
        self,
        endpoint: OfficialCratesIoEndpoint,
        success: JsonSuccess<'_>,
    ) -> Result<Self::Response, Error> {
        self.decode(endpoint, success)
    }
}
