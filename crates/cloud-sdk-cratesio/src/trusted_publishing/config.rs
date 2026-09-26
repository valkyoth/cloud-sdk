use super::TrustedPublishingError as Error;
use crate::{
    identifiers::CrateName,
    query::{FixedSegment, QueryOperation},
};
use cloud_sdk::buffer::SnapshotEncoder;

/// The two source-locked OIDC providers, never an arbitrary issuer URL.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Publisher {
    /// GitHub Actions.
    GitHub,
    /// GitLab.com CI/CD, not self-hosted GitLab.
    GitLab,
}
impl Publisher {
    /// Exact issuer expected in unverified preflight claims; not proof of signature.
    pub const fn issuer(self) -> &'static str {
        match self {
            Self::GitHub => "https://token.actions.githubusercontent.com",
            Self::GitLab => "https://gitlab.com",
        }
    }
    pub(super) const fn segment(self) -> FixedSegment {
        match self {
            Self::GitHub => FixedSegment::GithubConfigs,
            Self::GitLab => FixedSegment::GitlabConfigs,
        }
    }
    pub(super) const fn query(self) -> QueryOperation {
        match self {
            Self::GitHub => QueryOperation::GithubConfigs,
            Self::GitLab => QueryOperation::GitlabConfigs,
        }
    }
    pub(super) const fn field(self) -> &'static str {
        match self {
            Self::GitHub => "github_config",
            Self::GitLab => "gitlab_config",
        }
    }
    pub(super) const fn fields(self) -> [&'static str; 3] {
        match self {
            Self::GitHub => ["repository_owner", "repository_name", "workflow_filename"],
            Self::GitLab => ["namespace", "project", "workflow_filepath"],
        }
    }
}
/// Immutable crate-bound configuration intent. Names remain caller-owned.
/// Local syntax is conservative: no wildcards, controls, traversal or templates.
#[derive(Clone, Copy)]
pub struct PublisherConfig<'a> {
    pub(super) publisher: Publisher,
    pub(super) krate: CrateName<'a>,
    pub(super) owner: &'a str,
    pub(super) project: &'a str,
    pub(super) workflow: &'a str,
    pub(super) environment: Option<&'a str>,
}
impl<'a> PublisherConfig<'a> {
    /// Exact owner/namespace, repository/project and workflow; None deliberately
    /// permits any environment upstream. This does not prove repository ownership.
    pub fn new(
        publisher: Publisher,
        krate: CrateName<'a>,
        owner: &'a str,
        project: &'a str,
        workflow: &'a str,
        environment: Option<&'a str>,
    ) -> Result<Self, Error> {
        validate(publisher, owner, project, workflow, environment)?;
        Ok(Self {
            publisher,
            krate,
            owner,
            project,
            workflow,
            environment,
        })
    }
    /// Provider identity without private configuration text.
    pub const fn publisher(self) -> Publisher {
        self.publisher
    }
    pub(super) fn encode(self, e: &mut SnapshotEncoder<'_, Error>) -> Result<(), Error> {
        e.string("{")?;
        e.json_string(self.publisher.field())?;
        e.string(":{\"crate\":")?;
        e.json_string(self.krate.as_str())?;
        for (name, value) in
            self.publisher
                .fields()
                .into_iter()
                .zip([self.owner, self.project, self.workflow])
        {
            e.byte(b',')?;
            e.json_string(name)?;
            e.byte(b':')?;
            e.json_string(value)?;
        }
        e.string(",\"environment\":")?;
        match self.environment {
            Some(v) => e.json_string(v)?,
            None => e.string("null")?,
        }
        e.string("}}")
    }
}
impl core::fmt::Debug for PublisherConfig<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("PublisherConfig([redacted])")
    }
}
fn atom(s: &str) -> bool {
    !s.is_empty()
        && s != "."
        && s != ".."
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
fn gitlab_atom(s: &str) -> bool {
    atom(s)
        && s.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
        && s.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric)
        && !s.ends_with(".git")
        && !s.ends_with(".atom")
}
pub(super) fn validate(
    publisher: Publisher,
    owner: &str,
    project: &str,
    workflow: &str,
    environment: Option<&str>,
) -> Result<(), Error> {
    if [owner, project, workflow]
        .into_iter()
        .chain(environment)
        .any(|v| v.is_empty() || v.len() > 255)
    {
        return Err(Error::Limit);
    }
    let names = match publisher {
        Publisher::GitHub => {
            owner
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphanumeric)
                && owner
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                && atom(project)
        }
        Publisher::GitLab => owner.split('/').all(gitlab_atom) && gitlab_atom(project),
    };
    if !names
        || !(workflow.ends_with(".yml") || workflow.ends_with(".yaml"))
        || !workflow.split('/').all(atom)
        || workflow
            .rsplit('/')
            .next()
            .is_some_and(|s| s == ".yml" || s == ".yaml")
        || (publisher == Publisher::GitHub && workflow.contains('/'))
    {
        return Err(Error::Value);
    }
    if let Some(env) = environment
        && (env.trim() != env
            || env
                .chars()
                .any(|c| c.is_control() || "*?[]{}$\\'\"`,;".contains(c))
            || (publisher == Publisher::GitLab
                && !env
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b" -_/".contains(&b))))
    {
        return Err(Error::Value);
    }
    Ok(())
}
