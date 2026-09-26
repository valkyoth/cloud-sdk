use super::{
    Publisher, TemporaryToken, TrustedPublishingError as Error, TrustedPublishingPermit,
    request::Intent,
};
use crate::{
    discovery::{DiscoveryValue as Value, models::Timestamp, value::Builder},
    identifiers::{CrateName, NumericId},
    pagination::{Cursor, Direction, PageLink},
    query::{ApiPath, FixedSegment as F, Parameter, PathSegment as P},
    wire::JsonSuccess,
};
use alloc::vec::Vec;
use cloud_sdk::incremental_json::IncrementalJsonProgress;

/// Checked configuration metadata, not proof of ownership or future authority.
pub struct Configuration {
    publisher: Publisher,
    id: NumericId,
    fields: Value,
}
impl Configuration {
    /// Provider-qualified configuration ID.
    pub const fn id(&self) -> NumericId {
        self.id
    }
    /// OIDC provider owning this configuration.
    pub const fn publisher(&self) -> Publisher {
        self.publisher
    }
    /// Protected complete source fields, including provider numeric IDs.
    pub const fn fields(&self) -> &Value {
        &self.fields
    }
}
/// One bounded seek page; the caller controls traversal and fresh consent.
pub struct ConfigurationPage {
    entries: Vec<Configuration>,
    total: u64,
    meta: Value,
}
impl core::fmt::Debug for Configuration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Configuration([redacted])")
    }
}
impl core::fmt::Debug for ConfigurationPage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("ConfigurationPage([redacted])")
    }
}
impl ConfigurationPage {
    /// Checked, unique entries on this page.
    pub fn entries(&self) -> &[Configuration] {
        &self.entries
    }
    /// Upstream snapshot count, not a freshness guarantee.
    pub const fn total(&self) -> u64 {
        self.total
    }
    /// Lend the validated continuation text without letting protected borrows escape.
    /// It retains every original filter. Revalidate with PageLink/Query and use a
    /// new authenticated list permit; never send arbitrary metadata URLs.
    pub fn with_next_link<R>(&self, inspect: impl FnOnce(Option<&str>) -> R) -> Result<R, Error> {
        let value = self.meta.required("next_page")?;
        if value.is_null() {
            Ok(inspect(None))
        } else {
            value.with_text(|s| inspect(Some(s)))
        }
    }
}
/// Checked exact-operation result; deletion/revocation have no JSON body.
#[derive(Debug)]
pub enum TrustedPublishingResponse {
    /// One seek page of private configuration metadata.
    Configurations(ConfigurationPage),
    /// The server's checked configuration creation acknowledgement.
    Created(Configuration),
    /// Empty deletion acknowledgement; no additional existence proof.
    Deleted,
    /// Protected exchanged credential, locally time/crate bounded.
    Exchanged(TemporaryToken),
    /// Empty remote acknowledgement; local token was consumed and erased.
    Revoked,
}
impl TrustedPublishingPermit<'_> {
    /// Consumes authority while decoding a trusted, associated JSON exchange.
    /// `now` is fresh trusted Unix time. This does not execute any network request.
    pub fn decode_response(
        self,
        success: JsonSuccess<'_>,
        now: u64,
    ) -> Result<TrustedPublishingResponse, Error> {
        let origin = self.origin();
        let mut builder = Builder::default();
        if success
            .visit(&mut builder)
            .map_err(|e| e.into_visitor_error().unwrap_or(Error::Json))?
            != IncrementalJsonProgress::Complete
        {
            return Err(Error::Json);
        }
        let mut root = builder.finish()?;
        use super::schema_table as S;
        let index = match &self.0 {
            Intent::List(Publisher::GitHub, ..) => S::LIST_TRUSTPUB_GITHUB_CONFIGS,
            Intent::List(Publisher::GitLab, ..) => S::LIST_TRUSTPUB_GITLAB_CONFIGS,
            Intent::Create(c, _) if c.publisher == Publisher::GitHub => {
                S::CREATE_TRUSTPUB_GITHUB_CONFIG
            }
            Intent::Create(..) => S::CREATE_TRUSTPUB_GITLAB_CONFIG,
            Intent::Exchange(..) => S::EXCHANGE_TRUSTPUB_TOKEN,
            _ => return Err(Error::Binding),
        };
        crate::catalog::schema::validate_table(&root, index, 0, S::NODES)?;
        match self.0 {
            Intent::Exchange(_, policy) => {
                root.visit_fields(|key, _| {
                    if key == "token" {
                        Ok(())
                    } else {
                        Err(Error::Schema)
                    }
                })?;
                TemporaryToken::new(root.take("token")?, origin, policy, now)
                    .map(TrustedPublishingResponse::Exchanged)
            }
            Intent::Create(c, _) => {
                let config = configuration(c.publisher, root.take(c.publisher.field())?)?;
                super::oidc::equals(&config.fields, "crate", c.krate.as_str(), false)?;
                for ((key, expected), insensitive) in c
                    .publisher
                    .fields()
                    .into_iter()
                    .zip([c.owner, c.project, c.workflow])
                    .zip([c.publisher == Publisher::GitHub, false, false])
                {
                    super::oidc::equals(&config.fields, key, expected, insensitive)?;
                }
                match c.environment {
                    Some(v) => super::oidc::equals(&config.fields, "environment", v, false)?,
                    None if !config.fields.required("environment")?.is_null() => {
                        return Err(Error::Binding);
                    }
                    None => (),
                }
                Ok(TrustedPublishingResponse::Created(config))
            }
            Intent::List(p, query, _) => {
                let field = if p == Publisher::GitHub {
                    "github_configs"
                } else {
                    "gitlab_configs"
                };
                let raw = root.take(field)?.into_array()?;
                if u64::try_from(raw.len()).map_err(|_| Error::Limit)? > u64::from(query.per_page())
                {
                    return Err(Error::Limit);
                }
                let mut entries: Vec<Configuration> = Vec::new();
                entries
                    .try_reserve_exact(raw.len())
                    .map_err(|_| Error::Allocation)?;
                for value in raw {
                    let config = configuration(p, value)?;
                    for prior in &entries {
                        if prior.id == config.id || same_configuration(prior, &config)? {
                            return Err(Error::Binding);
                        }
                    }
                    for param in query.parameters() {
                        if let Parameter::Crate(name) = param {
                            super::oidc::equals(&config.fields, "crate", name.as_str(), false)?;
                        }
                    }
                    entries.push(config);
                }
                let meta = root.take("meta")?;
                let total = meta.required("total")?.count(i64::MAX as u64)?;
                if total < u64::try_from(entries.len()).map_err(|_| Error::Limit)? {
                    return Err(Error::Binding);
                }
                let next = meta.required("next_page")?;
                if !next.is_null() {
                    if entries.is_empty() {
                        return Err(Error::Binding);
                    }
                    next.with_text(|s| {
                        let segments = [P::Fixed(F::TrustedPublishing), P::Fixed(p.segment())];
                        let path = ApiPath::new(&segments).map_err(|_| Error::Binding)?;
                        let link =
                            PageLink::new(origin.endpoint(), path, query, s, Direction::Next)
                                .map_err(|_| Error::Binding)?;
                        if matches!(link.cursor(), Cursor::Seek(_)) {
                            Ok(())
                        } else {
                            Err(Error::Binding)
                        }
                    })??;
                }
                Ok(TrustedPublishingResponse::Configurations(
                    ConfigurationPage {
                        entries,
                        total,
                        meta,
                    },
                ))
            }
            _ => Err(Error::Binding),
        }
    }
}
fn configuration(publisher: Publisher, fields: Value) -> Result<Configuration, Error> {
    let id =
        NumericId::new(fields.required("id")?.count(i32::MAX as u64)?).map_err(|_| Error::Value)?;
    fields
        .required("crate")?
        .with_text(|s| CrateName::new(s).map(|_| ()).map_err(|_| Error::Value))??;
    let [a, b, c] = publisher.fields();
    fields.required(a)?.with_text(|a| {
        fields.required(b)?.with_text(|b| {
            fields.required(c)?.with_text(|c| {
                let environment = fields.required("environment")?;
                if environment.is_null() {
                    super::config::validate(publisher, a, b, c, None)
                } else {
                    environment
                        .with_text(|env| super::config::validate(publisher, a, b, c, Some(env)))?
                }
            })?
        })?
    })??;
    Timestamp::parse(fields.required("created_at")?)?;
    match publisher {
        Publisher::GitHub => {
            NumericId::new(
                fields
                    .required("repository_owner_id")?
                    .count(i32::MAX as u64)?,
            )
            .map_err(|_| Error::Value)?;
        }
        Publisher::GitLab => {
            let value = fields.required("namespace_id")?;
            if !value.is_null() {
                value.with_text(|s| {
                    if !s.is_empty()
                        && s.len() <= 20
                        && s.bytes().all(|b| b.is_ascii_digit())
                        && s.parse::<u64>().ok().filter(|n| *n > 0).is_some()
                    {
                        Ok(())
                    } else {
                        Err(Error::Value)
                    }
                })??;
            }
        }
    }
    Ok(Configuration {
        publisher,
        id,
        fields,
    })
}
fn same_configuration(a: &Configuration, b: &Configuration) -> Result<bool, Error> {
    for field in ["crate", "environment"]
        .into_iter()
        .chain(a.publisher.fields())
    {
        let x = a.fields.required(field)?;
        let y = b.fields.required(field)?;
        let equal = if x.is_null() || y.is_null() {
            x.is_null() && y.is_null()
        } else {
            x.with_text(|x| y.with_text(|y| x == y))??
        };
        if !equal {
            return Ok(false);
        }
    }
    Ok(true)
}
