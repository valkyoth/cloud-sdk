use super::{
    EndpointScope, MAX_TOKEN_SCOPES, TokenMetadata, TokenOperation, TokenPermit, TokenResponse,
};
use crate::{
    discovery::{DiscoveryError as Error, value::Builder},
    wire::JsonSuccess,
};
use alloc::vec::Vec;
use cloud_sdk::incremental_json::IncrementalJsonProgress;

impl TokenPermit<'_> {
    /// Decode an admitted JSON response associated with this exact exchange.
    /// Self-revocation's empty 204 is handled only by the execution boundary.
    pub fn decode_response(&self, success: JsonSuccess<'_>) -> Result<TokenResponse, Error> {
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
            TokenOperation::RevokeCurrent => Err(Error::Binding),
            TokenOperation::Revoke => {
                // Source controller returns precisely {}; no contradictory fields.
                root.visit_fields(|_, _| Err(Error::Schema))?;
                Ok(TokenResponse::Revoked)
            }
            TokenOperation::Inspect => {
                crate::catalog::schema::validate_table(
                    &root,
                    super::schema_table::FIND_API_TOKEN,
                    0,
                    super::schema_table::NODES,
                )?;
                let fields = root.take("api_token")?;
                let id = crate::identifiers::NumericId::new(
                    fields.required("id")?.count(i32::MAX as u64)?,
                )
                .map_err(|_| Error::Value)?;
                if self.id != Some(id) {
                    return Err(Error::Binding);
                }
                fields.required("name")?.with_text(|v| {
                    if v.len() <= 1024 {
                        Ok(())
                    } else {
                        Err(Error::Limit)
                    }
                })??;
                let crates = fields.required("crate_scopes")?;
                if !crates.is_null() {
                    let values = crates.array()?;
                    if values.len() > MAX_TOKEN_SCOPES {
                        return Err(Error::Limit);
                    }
                    for value in values {
                        value.with_text(|s| {
                            if s.is_empty() || s.len() > 256 || s.chars().any(char::is_control) {
                                Err(Error::Value)
                            } else {
                                Ok(())
                            }
                        })??;
                    }
                }
                let scopes = fields.required("endpoint_scopes")?;
                let endpoints = if scopes.is_null() {
                    None
                } else {
                    let values = scopes.array()?;
                    if values.len() > MAX_TOKEN_SCOPES {
                        return Err(Error::Limit);
                    }
                    let mut parsed = Vec::new();
                    parsed
                        .try_reserve_exact(values.len())
                        .map_err(|_| Error::Allocation)?;
                    for value in values {
                        parsed.push(value.with_text(EndpointScope::parse)??);
                    }
                    Some(parsed)
                };
                Ok(TokenResponse::Metadata(TokenMetadata {
                    id,
                    fields,
                    endpoints,
                }))
            }
        }
    }
}
