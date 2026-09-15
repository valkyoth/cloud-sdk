use super::{DownloadError as Error, DownloadOperation as Op, *};
use crate::{
    discovery::{DiscoveryValue as Value, value::Builder},
    endpoint::{OfficialCratesIoEndpoint, OfficialEndpointPurpose},
    identifiers::{CrateName, Date, Version},
    query::{Include, Page, Parameter},
    wire::JsonSuccess,
};
use cloud_sdk::incremental_json::IncrementalJsonProgress;

impl DownloadRequest<'_> {
    /// Decode admitted JSON; prefer DownloadClient to bind execution and decoding.
    pub fn decode(
        self,
        endpoint: OfficialCratesIoEndpoint,
        success: JsonSuccess<'_>,
    ) -> Result<DownloadResponse, Error> {
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
        let root = builder.finish()?;
        let schema = match self.operation {
            Op::Location => schema_table::DOWNLOAD_VERSION,
            Op::CrateCounts => schema_table::GET_CRATE_DOWNLOADS,
            Op::VersionCounts => schema_table::GET_VERSION_DOWNLOADS,
            Op::ReverseDependencies => schema_table::LIST_REVERSE_DEPENDENCIES,
        };
        crate::catalog::schema::validate_table(&root, schema, 0, schema_table::NODES)?;
        match self.operation {
            Op::Location => {
                root.required("url")?.with_text(|s| {
                    if s.is_empty()
                        || s.len() > crate::endpoint::MAX_DOWNLOAD_REDIRECT_LOCATION_BYTES
                        || s.chars().any(char::is_control)
                    {
                        Err(Error::Value)
                    } else {
                        Ok(())
                    }
                })??;
                Ok(DownloadResponse::Location(DownloadLocation(root)))
            }
            Op::CrateCounts | Op::VersionCounts => {
                self.counts(&root)?;
                Ok(DownloadResponse::Counts(DownloadCounts(root)))
            }
            Op::ReverseDependencies => self.reverse(root),
        }
    }
    fn counts(self, root: &Value) -> Result<(), Error> {
        let buckets = root.required("version_downloads")?.array()?;
        let version_only = self.operation == Op::VersionCounts;
        if buckets.len() > if version_only { 90 } else { 450 } {
            return Err(Error::Limit);
        }
        let mut first_id = None;
        let mut ids = [0_u64; 5];
        let mut id_count = 0_usize;
        let mut first_day = None;
        let mut last_day = None;
        for (i, bucket) in buckets.iter().enumerate() {
            let id = positive(bucket.required("version")?)?;
            if !ids.contains(&id) {
                *ids.get_mut(id_count).ok_or(Error::Limit)? = id;
                id_count = id_count.checked_add(1).ok_or(Error::Limit)?;
            }
            if version_only && first_id.is_some_and(|v| v != id) {
                return Err(Error::Binding);
            }
            first_id = Some(id);
            bucket.required("downloads")?.count(i32::MAX as u64)?;
            date(bucket.required("date")?)?;
            let day = bucket.required("date")?.with_text(ordinal)??;
            first_day = Some(first_day.map_or(day, |n: u32| n.min(day)));
            last_day = Some(last_day.map_or(day, |n: u32| n.max(day)));
            if let Some(query) = self.query {
                for parameter in query.parameters() {
                    if let Parameter::BeforeDate(before) = parameter {
                        let end = ordinal(before.as_str())?;
                        if end.checked_sub(day).is_none_or(|span| span > 89) {
                            return Err(Error::Binding);
                        }
                    }
                }
            }
            for prior in buckets.get(..i).ok_or(Error::Limit)? {
                if positive(prior.required("version")?)? == id && same_date(bucket, prior)? {
                    return Err(Error::Binding);
                }
            }
        }
        if first_day
            .zip(last_day)
            .is_some_and(|(start, end)| end.saturating_sub(start) > 89)
        {
            return Err(Error::Binding);
        }
        if !version_only {
            let extras = root
                .required("meta")?
                .required("extra_downloads")?
                .array()?;
            if extras.len() > 90 {
                return Err(Error::Limit);
            }
            for (i, bucket) in extras.iter().enumerate() {
                date(bucket.required("date")?)?;
                let day = bucket.required("date")?.with_text(ordinal)??;
                first_day = Some(first_day.map_or(day, |n: u32| n.min(day)));
                last_day = Some(last_day.map_or(day, |n: u32| n.max(day)));
                bucket.required("downloads")?.count(i64::MAX as u64)?;
                for prior in extras.get(..i).ok_or(Error::Limit)? {
                    if same_date(bucket, prior)? {
                        return Err(Error::Binding);
                    }
                }
            }
            if first_day
                .zip(last_day)
                .is_some_and(|(start, end)| end.saturating_sub(start) > 89)
            {
                return Err(Error::Binding);
            }
            let include = self.query.ok_or(Error::Binding)?.parameters().iter().any(
                |p| matches!(p, Parameter::Include(s) if s.values().contains(&Include::Versions)),
            );
            if root.get("versions")?.is_some() != include {
                return Err(Error::Binding);
            }
            if let Some(versions) = root.get("versions")? {
                let versions = versions.array()?;
                if versions.len() > 5 {
                    return Err(Error::Limit);
                }
                records(versions, Some(self.name))?;
                for bucket in buckets {
                    let id = positive(bucket.required("version")?)?;
                    if !contains_id(versions, id)? {
                        return Err(Error::Binding);
                    }
                }
            }
        }
        Ok(())
    }
    fn reverse(self, root: Value) -> Result<DownloadResponse, Error> {
        let query = self.query.ok_or(Error::Binding)?;
        let deps = root.required("dependencies")?.array()?;
        let versions = root.required("versions")?.array()?;
        if deps.len() > query.per_page() as usize || versions.len() > deps.len() {
            return Err(Error::Limit);
        }
        records(versions, None)?;
        for (i, dep) in deps.iter().enumerate() {
            let id = positive(dep.required("id")?)?;
            for prior in deps.get(..i).ok_or(Error::Limit)? {
                if positive(prior.required("id")?)? == id {
                    return Err(Error::Binding);
                }
            }
            if !dep
                .required("crate_id")?
                .with_text(|s| equivalent(s, self.name.as_str()))?
            {
                return Err(Error::Binding);
            }
            if !contains_id(versions, positive(dep.required("version_id")?)?)? {
                return Err(Error::Binding);
            }
            text(dep.required("req")?, 4096)?;
            text(dep.required("kind")?, 64)?;
            if !dep.required("target")?.is_null() {
                text(dep.required("target")?, 4096)?;
            }
            for feature in dep.required("features")?.array()? {
                text(feature, 256)?;
            }
            dep.required("downloads")?.count(i64::MAX as u64)?;
        }
        for version in versions {
            let id = positive(version.required("id")?)?;
            let mut found = false;
            for dep in deps {
                found |= positive(dep.required("version_id")?)? == id;
            }
            if !found {
                return Err(Error::Binding);
            }
        }
        let total = root
            .required("meta")?
            .required("total")?
            .count(i64::MAX as u64)?;
        let current = query.page().map(Page::get).unwrap_or(1);
        let offset = u64::from(current.checked_sub(1).ok_or(Error::Limit)?)
            .checked_mul(u64::from(query.per_page()))
            .ok_or(Error::Limit)?;
        if total < deps.len() as u64 || (offset < total && deps.is_empty()) {
            return Err(Error::Binding);
        }
        if !deps.is_empty() && offset.checked_add(deps.len() as u64).ok_or(Error::Limit)? > total {
            return Err(Error::Binding);
        }
        let next = if offset.checked_add(deps.len() as u64).ok_or(Error::Limit)? >= total {
            ReverseContinuation::End
        } else if deps.len() != query.per_page() as usize {
            return Err(Error::Binding);
        } else if current == crate::query::MAX_PAGE {
            ReverseContinuation::LimitReached
        } else {
            ReverseContinuation::Next(
                Page::new(current.checked_add(1).ok_or(Error::Limit)?).map_err(|_| Error::Limit)?,
            )
        };
        Ok(DownloadResponse::ReverseDependencies(ReverseDependencies {
            fields: root,
            next,
        }))
    }
}
fn positive(value: &Value) -> Result<u64, Error> {
    let value = value.count(i32::MAX as u64)?;
    if value == 0 {
        Err(Error::Value)
    } else {
        Ok(value)
    }
}
fn equivalent(a: &str, b: &str) -> bool {
    let map = |b: u8| {
        if b == b'_' {
            b'-'
        } else {
            b.to_ascii_lowercase()
        }
    };
    a.bytes().map(map).eq(b.bytes().map(map))
}
fn text(value: &Value, max: usize) -> Result<(), Error> {
    value.with_text(|s| {
        if s.is_empty() || s.len() > max || s.chars().any(char::is_control) {
            Err(Error::Value)
        } else {
            Ok(())
        }
    })?
}
fn date(value: &Value) -> Result<(), Error> {
    value.with_text(|s| Date::new(s).map(|_| ()).map_err(|_| Error::Value))?
}
fn same_date(a: &Value, b: &Value) -> Result<bool, Error> {
    a.required("date")?
        .with_text(|a| b.required("date")?.with_text(|b| a == b))?
}
fn ordinal(text: &str) -> Result<u32, Error> {
    Date::new(text).map_err(|_| Error::Value)?;
    let mut parts = text.split('-');
    let mut number = || {
        parts
            .next()
            .ok_or(Error::Value)?
            .parse::<u32>()
            .map_err(|_| Error::Value)
    };
    let year = number()?;
    let month = number()?;
    let day = number()?;
    let previous = year.checked_sub(1).ok_or(Error::Value)?;
    let mut days = previous
        .checked_mul(365)
        .and_then(|n| n.checked_add(previous / 4))
        .and_then(|n| n.checked_sub(previous / 100))
        .and_then(|n| n.checked_add(previous / 400))
        .and_then(|n| n.checked_add(day))
        .ok_or(Error::Limit)?;
    for m in 1..month {
        let length = match m {
            4 | 6 | 9 | 11 => 30,
            2 if year.is_multiple_of(400)
                || year.is_multiple_of(4) && !year.is_multiple_of(100) =>
            {
                29
            }
            2 => 28,
            _ => 31,
        };
        days = days.checked_add(length).ok_or(Error::Limit)?;
    }
    Ok(days)
}
fn contains_id(values: &[Value], id: u64) -> Result<bool, Error> {
    for value in values {
        if positive(value.required("id")?)? == id {
            return Ok(true);
        }
    }
    Ok(false)
}
fn records(values: &[Value], name: Option<CrateName<'_>>) -> Result<(), Error> {
    for (i, value) in values.iter().enumerate() {
        let id = positive(value.required("id")?)?;
        if contains_id(values.get(..i).ok_or(Error::Limit)?, id)? {
            return Err(Error::Binding);
        }
        value.required("crate")?.with_text(|s| {
            CrateName::new(s).map_err(|_| Error::Value)?;
            if name.is_some_and(|n| !equivalent(n.as_str(), s)) {
                Err(Error::Binding)
            } else {
                Ok(())
            }
        })??;
        value
            .required("num")?
            .with_text(|s| Version::new(s).map(|_| ()).map_err(|_| Error::Value))??;
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
                text(value, 256)?;
            }
            Ok(())
        })?;
    }
    Ok(())
}
#[cfg(any(feature = "blocking", feature = "async"))]
impl crate::discovery::checked::CheckedGet for DownloadRequest<'_> {
    type Response = DownloadResponse;
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
