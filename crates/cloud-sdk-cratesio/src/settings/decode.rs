use super::{SettingsError as Error, SettingsPermit, YankMessage, request::Patch};
use crate::{
    discovery::{SummaryCrate, crate_model::crate_record, value::Builder},
    versions::{VersionRecord, VersionRequest, VersionResponse},
    wire::JsonSuccess,
};
use cloud_sdk::incremental_json::IncrementalJsonProgress;

/// Checked postcondition at the returned snapshot, not a concurrency lock or
/// proof that asynchronous index synchronization has finished.
#[derive(Debug)]
// As with CatalogResponse, keep bounded metadata inline rather than add an
// infallible allocation solely to reduce enum size.
#[allow(clippy::large_enum_variant)]
pub enum SettingsResponse {
    /// Updated metadata bound to the requested crate and trustpub policy.
    Crate(SummaryCrate),
    /// Updated exact version, bound to requested yank state and message.
    Version(VersionRecord),
}
impl SettingsPermit<'_> {
    /// Decode the admitted response from this exact trusted transport exchange.
    /// A mismatch may occur after mutation; errors never authorize automatic retry.
    pub fn decode_response(&self, success: JsonSuccess<'_>) -> Result<SettingsResponse, Error> {
        match self.request.patch {
            Patch::Version(version, yanked, message) => {
                let endpoint = match self.credential.origin() {
                    crate::credentials::CredentialOrigin::Production => {
                        crate::endpoint::OfficialCratesIoEndpoint::production_api()
                    }
                    crate::credentials::CredentialOrigin::Staging => {
                        crate::endpoint::OfficialCratesIoEndpoint::staging_api()
                    }
                };
                let VersionResponse::Version(record) =
                    VersionRequest::detail(self.request.name, version).decode(endpoint, success)?
                else {
                    return Err(Error::Binding);
                };
                let actual = record.fields().required("yanked")?.boolean()?;
                if yanked.is_some_and(|wanted| wanted != actual) {
                    return Err(Error::Binding);
                }
                let returned = record.fields().required("yank_message")?;
                match message {
                    YankMessage::Clear if !returned.is_null() => return Err(Error::Binding),
                    YankMessage::Set(text) => {
                        if !actual || !returned.with_text(|value| value == text)? {
                            return Err(Error::Binding);
                        }
                    }
                    YankMessage::Clear => {}
                }
                Ok(SettingsResponse::Version(record))
            }
            Patch::Crate(enabled) => {
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
                    super::schema_table::UPDATE_CRATE,
                    0,
                    super::schema_table::NODES,
                )?;
                let record = crate_record(&mut root.take("crate")?)?;
                let canonical = |b: u8| {
                    if b == b'_' {
                        b'-'
                    } else {
                        b.to_ascii_lowercase()
                    }
                };
                for returned in [&record.id, &record.name] {
                    if !self
                        .request
                        .name
                        .as_str()
                        .bytes()
                        .map(canonical)
                        .eq(returned.bytes().map(canonical))
                    {
                        return Err(Error::Binding);
                    }
                }
                if record.trustpub_only != enabled {
                    return Err(Error::Binding);
                }
                Ok(SettingsResponse::Crate(record))
            }
        }
    }
}
