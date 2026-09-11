use super::{CatalogError as Error, CatalogOperation as Op, *};
use crate::{
    discovery::{DiscoveryValue as Value, crate_model::crate_record, models::*, value::Builder},
    endpoint::{OfficialCratesIoEndpoint, OfficialEndpointPurpose},
    query::{Include, Parameter},
    wire::JsonSuccess,
};
use alloc::vec::Vec;
use cloud_sdk::incremental_json::IncrementalJsonProgress;

impl CatalogRequest<'_> {
    /// Decodes an already wire-admitted success, clearing its response storage.
    /// Use CatalogClient to bind dispatch, wire policy and decoding together.
    pub fn decode(
        self,
        endpoint: OfficialCratesIoEndpoint,
        success: JsonSuccess<'_>,
    ) -> Result<CatalogResponse, Error> {
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
        match self.operation {
            Op::List => {
                let (total, next, previous) =
                    super::pagination::links(self, endpoint, root.required("meta")?)?;
                let values = root.take("crates")?.into_array()?;
                let items = owned_list(values, self.query.per_page() as usize, |mut v| {
                    crate_record(&mut v)
                })?;
                Ok(CatalogResponse::Crates(CratePage {
                    items,
                    total,
                    next,
                    previous,
                }))
            }
            Op::CargoSearch => {
                let items = list(
                    root.required("crates")?,
                    self.query.per_page() as usize,
                    |v| {
                        Ok(CargoSearchCrate {
                            name: text(v, "name", 64)?,
                            max_version: text(v, "max_version", 150)?,
                            description: v
                                .get("description")?
                                .map(|v| nullable(v, |v| v.text(65_536)))
                                .transpose()?
                                .flatten(),
                        })
                    },
                )?;
                Ok(CatalogResponse::CargoSearch(CargoSearchPage {
                    items,
                    total: count(root.required("meta")?, "total")?,
                }))
            }
            Op::Crate | Op::NewCrate => self.metadata(&mut root).map(CatalogResponse::Crate),
        }
    }
    fn metadata(self, root: &mut Value) -> Result<CrateMetadata, Error> {
        let mut krate = root.take("crate")?;
        let krate = crate_record(&mut krate)?;
        let wanted = self.name.ok_or(Error::Binding)?.as_str();
        if !equivalent_name(wanted, &krate.name) || !equivalent_name(wanted, &krate.id) {
            return Err(Error::Binding);
        }
        let versions = root.take("versions")?;
        let versions = if versions.is_null() {
            None
        } else {
            Some(owned_list(versions.into_array()?, 1024, |v| {
                super::schema::validate(&v, super::schema_table::VERSION, 0)?;
                if !v
                    .required("crate")?
                    .with_text(|name| equivalent_name(wanted, name))?
                {
                    return Err(Error::Binding);
                }
                v.required("num")?.with_text(|v| {
                    crate::identifiers::Version::new(v)
                        .map(|_| ())
                        .map_err(|_| Error::Value)
                })??;
                let id = v.required("id")?.count(i32::MAX as u64)?;
                if id == 0 {
                    return Err(Error::Value);
                }
                if !v
                    .required("checksum")?
                    .with_text(|v| v.len() == 64 && v.bytes().all(|b| b.is_ascii_hexdigit()))?
                {
                    return Err(Error::Value);
                }
                Ok(IncludedVersion(v))
            })?)
        };
        let keywords = nullable(root.required("keywords")?, |v| list(v, 1024, keyword))?;
        let categories = nullable(root.required("categories")?, |v| {
            list(v, 1024, |v| category(v, 0))
        })?;
        // The controller's include selector is a response contract, not a hint.
        let includes = self.query.parameters().iter().find_map(|p| match p {
            Parameter::Include(s) => Some(s.values()),
            _ => None,
        });
        let selected =
            |which| includes.is_none_or(|s| s.contains(&Include::Full) || s.contains(&which));
        if keywords.is_some() != selected(Include::Keywords)
            || categories.is_some() != selected(Include::Categories)
        {
            return Err(Error::Binding);
        }
        let full_versions = selected(Include::Versions);
        let default_version =
            selected(Include::DefaultVersion) && !full_versions && krate.default_version.is_some();
        if versions.is_some() != (full_versions || default_version) {
            return Err(Error::Binding);
        }
        if default_version {
            let values = versions.as_ref().ok_or(Error::Binding)?;
            if values.len() != 1
                || !values
                    .first()
                    .ok_or(Error::Binding)?
                    .0
                    .required("num")?
                    .with_text(|num| krate.default_version.as_deref() == Some(num))?
            {
                return Err(Error::Binding);
            }
        }
        Ok(CrateMetadata {
            krate,
            versions,
            keywords,
            categories,
        })
    }
}
fn owned_list<T>(
    values: Vec<Value>,
    max: usize,
    mut parse: impl FnMut(Value) -> Result<T, Error>,
) -> Result<Vec<T>, Error> {
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
fn equivalent_name(left: &str, right: &str) -> bool {
    fn canonical(byte: u8) -> u8 {
        if byte == b'_' {
            b'-'
        } else {
            byte.to_ascii_lowercase()
        }
    }
    left.bytes().map(canonical).eq(right.bytes().map(canonical))
}
#[cfg(any(feature = "blocking", feature = "async"))]
impl crate::discovery::checked::CheckedGet for CatalogRequest<'_> {
    type Response = CatalogResponse;
    fn target(
        self,
        output: &mut [u8],
    ) -> Result<crate::endpoint::ApiRequestTarget<'_>, crate::query::QueryError> {
        self.write_target(output)
    }
    fn anonymous(self) -> Result<(), Error> {
        if self.following() {
            Err(Error::Binding)
        } else {
            Ok(())
        }
    }
    fn decode(
        self,
        endpoint: OfficialCratesIoEndpoint,
        success: JsonSuccess<'_>,
    ) -> Result<CatalogResponse, Error> {
        self.decode(endpoint, success)
    }
}
