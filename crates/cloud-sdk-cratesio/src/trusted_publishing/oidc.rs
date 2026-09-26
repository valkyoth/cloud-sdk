use super::{Publisher, PublisherConfig, TrustedPublishingError as Error};
use crate::discovery::{DiscoveryValue as Value, value::Builder};
use cloud_sdk::incremental_json::{
    IncrementalJsonDecoder, IncrementalJsonLimits, IncrementalJsonProgress,
};
use cloud_sdk_sanitization::{SecretBoxBytes, sanitize_bytes};

const JWT_DECODE_SCRATCH: usize = 12_288;

/// Caller-selected expectations for unverified JWT preflight, NOT authentication.
/// The registry alone validates signatures, keys, issuer authority, scope and replay.
#[derive(Clone, Copy)]
pub struct ExchangePolicy<'a> {
    pub(super) config: PublisherConfig<'a>,
    audience: &'a str,
    pub(super) started: u64,
    pub(super) deadline: u64,
}
impl<'a> ExchangePolicy<'a> {
    /// Use trusted local configuration and Unix time. A local lifetime of at most
    /// 1800 seconds starts here, not after an arbitrarily delayed exchange.
    /// Production normally uses audience `crates.io`; staging is operator-configured.
    pub fn new(
        config: PublisherConfig<'a>,
        audience: &'a str,
        now: u64,
        lifetime_seconds: u64,
    ) -> Result<Self, Error> {
        if audience.is_empty()
            || audience.len() > 255
            || audience
                .chars()
                .any(|c| c.is_control() || c.is_whitespace())
            || lifetime_seconds == 0
            || lifetime_seconds > 1800
        {
            return Err(Error::Value);
        }
        Ok(Self {
            config,
            audience,
            started: now,
            deadline: now.checked_add(lifetime_seconds).ok_or(Error::Limit)?,
        })
    }
    pub(super) fn time(self, now: u64) -> Result<(), Error> {
        if now < self.started || now >= self.deadline {
            Err(Error::Binding)
        } else {
            Ok(())
        }
    }
    /// Reject obvious local misrouting before network use. Unsigned claims can
    /// be forged; passing this check never proves authenticity or permission.
    /// Input is the bounded exchange JSON object containing `jwt`, not a raw
    /// compact JWT. The caller retains responsibility for erasing input storage.
    pub fn preflight(self, json: &[u8], now: u64) -> Result<(), Error> {
        self.preflight_allocating(json, now, || {
            SecretBoxBytes::try_zeroed(JWT_DECODE_SCRATCH, JWT_DECODE_SCRATCH)
                .map_err(|_| Error::Allocation)
        })
    }
    pub(super) fn preflight_allocating(
        self,
        json: &[u8],
        now: u64,
        allocate: impl FnOnce() -> Result<SecretBoxBytes, Error>,
    ) -> Result<(), Error> {
        self.time(now)?;
        let root = parse(json)?;
        let mut scratch = allocate()?;
        scratch.with_secret_mut(|scratch| {
            root.required("jwt")?.with_text(|jwt| {
                if jwt.len() > 16_384 {
                    return Err(Error::Limit);
                }
                let mut parts = jwt.split('.');
                let header = part(parts.next().ok_or(Error::Value)?, scratch)?;
                equals(&header, "alg", "RS256", false)?;
                required_text(&header, "kid")?;
                if header.get("crit")?.is_some() || header.get("b64")?.is_some() {
                    return Err(Error::Value);
                }
                let claims = part(parts.next().ok_or(Error::Value)?, scratch)?;
                let signature = parts.next().ok_or(Error::Value)?;
                sanitize_bytes(scratch);
                let length = base64_ng::ct::URL_SAFE_NO_PAD
                    .decode_slice_clear_tail(signature.as_bytes(), scratch)
                    .map_err(|_| Error::Value)?;
                if length == 0 || parts.next().is_some() {
                    return Err(Error::Value);
                }
                equals(&claims, "iss", self.config.publisher.issuer(), false)?;
                equals(&claims, "aud", self.audience, false)?;
                let issued = claims.required("iat")?.count(u64::MAX)?;
                let expires = claims.required("exp")?.count(u64::MAX)?;
                if issued > now || expires <= now || expires <= issued {
                    return Err(Error::Binding);
                }
                if let Some(nbf) = claims.get("nbf")?
                    && nbf.count(u64::MAX)? > now
                {
                    return Err(Error::Binding);
                }
                required_text(&claims, "jti")?;
                required_text(&claims, "sha")?;
                self.publisher_claims(&claims)
            })?
        })
    }
    fn publisher_claims(self, claims: &Value) -> Result<(), Error> {
        let c = self.config;
        let (repository, workflow, id, run) = match c.publisher {
            Publisher::GitHub => (
                "repository",
                "workflow_ref",
                "repository_owner_id",
                "run_id",
            ),
            Publisher::GitLab => (
                "project_path",
                "ci_config_ref_uri",
                "namespace_id",
                "job_id",
            ),
        };
        required_text(claims, run)?;
        claims.required(id)?.with_text(|s| {
            if s.is_empty()
                || s.len() > 20
                || !s.bytes().all(|b| b.is_ascii_digit())
                || s.parse::<u64>().ok().filter(|n| *n > 0).is_none()
            {
                Err(Error::Value)
            } else {
                Ok(())
            }
        })??;
        claims.required(repository)?.with_text(|s| {
            let (owner, project) = s.rsplit_once('/').ok_or(Error::Binding)?;
            if owner.eq_ignore_ascii_case(c.owner) && project.eq_ignore_ascii_case(c.project) {
                Ok(())
            } else {
                Err(Error::Binding)
            }
        })??;
        claims.required(workflow)?.with_text(|s| {
            let (path, reference) = s.rsplit_once('@').ok_or(Error::Binding)?;
            if reference.is_empty() || reference.chars().any(char::is_control) {
                return Err(Error::Value);
            }
            let (repo, file) = match c.publisher {
                Publisher::GitHub => path.split_once("/.github/workflows/"),
                Publisher::GitLab => path
                    .strip_prefix("gitlab.com/")
                    .and_then(|s| s.split_once("//")),
            }
            .ok_or(Error::Binding)?;
            let (owner, project) = repo.rsplit_once('/').ok_or(Error::Binding)?;
            if !owner.eq_ignore_ascii_case(c.owner)
                || !project.eq_ignore_ascii_case(c.project)
                || file != c.workflow
            {
                return Err(Error::Binding);
            }
            Ok(())
        })??;
        if c.publisher == Publisher::GitHub {
            claims.required("event_name")?.with_text(|s| {
                if s.is_empty() || matches!(s, "pull_request_target" | "workflow_run") {
                    Err(Error::Value)
                } else {
                    Ok(())
                }
            })??;
        }
        if let Some(environment) = c.environment {
            // ASCII comparison is intentionally stricter than upstream Unicode folding.
            equals(claims, "environment", environment, true)?;
        }
        Ok(())
    }
}
impl core::fmt::Debug for ExchangePolicy<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("ExchangePolicy([redacted])")
    }
}
fn required_text(v: &Value, field: &str) -> Result<(), Error> {
    v.required(field)?.with_text(|s| {
        if !s.is_empty() && s.len() <= 1024 && !s.chars().any(char::is_control) {
            Ok(())
        } else {
            Err(Error::Value)
        }
    })?
}
pub(super) fn equals(
    v: &Value,
    field: &str,
    expected: &str,
    insensitive: bool,
) -> Result<(), Error> {
    v.required(field)?.with_text(|s| {
        if s == expected || (insensitive && s.eq_ignore_ascii_case(expected)) {
            Ok(())
        } else {
            Err(Error::Binding)
        }
    })?
}
fn part(text: &str, storage: &mut [u8]) -> Result<Value, Error> {
    sanitize_bytes(storage);
    let n = base64_ng::ct::URL_SAFE_NO_PAD
        .decode_slice_clear_tail(text.as_bytes(), storage)
        .map_err(|_| Error::Value)?;
    parse(storage.get(..n).ok_or(Error::Limit)?)
}
pub(super) fn parse(bytes: &[u8]) -> Result<Value, Error> {
    let mut builder = Builder::default();
    let limits = IncrementalJsonLimits::DEFAULT
        .with_input_bytes(16_400)
        .map_err(|_| Error::Limit)?;
    let mut decoder = IncrementalJsonDecoder::with_limits(limits);
    decoder
        .push(bytes, &mut builder)
        .map_err(|e| e.into_visitor_error().unwrap_or(Error::Json))?;
    if decoder
        .finish(&mut builder)
        .map_err(|e| e.into_visitor_error().unwrap_or(Error::Json))?
        != IncrementalJsonProgress::Complete
    {
        return Err(Error::Json);
    }
    builder.finish()
}
