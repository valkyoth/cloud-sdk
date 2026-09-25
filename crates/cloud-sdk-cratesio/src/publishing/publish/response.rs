use super::{PublishError as Error, PublishPermit};
use crate::{
    discovery::{DiscoveryValue, SummaryCrate, crate_model::crate_record, value::Builder},
    wire::JsonSuccess,
};
use cloud_sdk::incremental_json::IncrementalJsonProgress;

/// Checked crates.io publication acknowledgement, not proof of index visibility.
#[derive(Debug)]
pub struct PublishResponse {
    krate: SummaryCrate,
    warnings: DiscoveryValue,
}
impl PublishResponse {
    /// Returned named crate metadata. The reply does not echo the uploaded version.
    pub const fn krate(&self) -> &SummaryCrate {
        &self.krate
    }
    /// Inert protected warning lists: invalid_categories, invalid_badges, other.
    pub const fn warnings(&self) -> &DiscoveryValue {
        &self.warnings
    }
}
impl PublishPermit<'_> {
    /// Decode this trusted exchange's exact-200 wire-admitted response. Even a
    /// decoding error can follow publication; never automatically retry.
    pub fn decode_response(self, success: JsonSuccess<'_>) -> Result<PublishResponse, Error> {
        let mut builder = Builder::default();
        if success
            .visit(&mut builder)
            .map_err(|e| e.into_visitor_error().unwrap_or(Error::Json))?
            != IncrementalJsonProgress::Complete
        {
            return Err(Error::Json);
        }
        let mut root = builder.finish()?;
        crate::catalog::schema::validate_table(
            &root,
            super::schema_table::PUBLISH,
            0,
            super::schema_table::NODES,
        )?;
        let krate = crate_record(&mut root.take("crate")?)?;
        for value in [&krate.name, &krate.id] {
            if !self
                .request
                .metadata
                .fields
                .required("name")?
                .with_text(|n| n == value)?
            {
                return Err(Error::Binding);
            }
        }
        let warnings = root.take("warnings")?;
        for field in ["invalid_categories", "invalid_badges", "other"] {
            let values = warnings.required(field)?.array()?;
            if values.len() > 64 {
                return Err(Error::Limit);
            }
            for value in values {
                super::validation::text(value, 4096, |_| true)?;
            }
        }
        Ok(PublishResponse { krate, warnings })
    }
}
