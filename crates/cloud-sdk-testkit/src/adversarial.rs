//! Reusable adversarial response corpus.

use crate::{FixtureBody, FixtureBodyError};

/// Common response limit used by the initial provider response boundary.
pub const DEFAULT_RESPONSE_LIMIT: usize = 8_388_608;

/// Adversarial response category.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AdversarialKind {
    /// Syntactically malformed JSON.
    MalformedJson,
    /// Valid JSON containing an additive unknown field.
    UnknownFields,
    /// Valid JSON missing required fields.
    MissingRequiredFields,
    /// Response one byte larger than the common admitted ceiling.
    OversizedResponse,
    /// Structurally invalid pagination values.
    InvalidPagination,
    /// Unknown action lifecycle state.
    InvalidActionState,
}

/// Named adversarial response body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdversarialFixture<'a> {
    kind: AdversarialKind,
    body: FixtureBody<'a>,
}

impl<'a> AdversarialFixture<'a> {
    /// Creates a named adversarial fixture.
    #[must_use]
    pub const fn new(kind: AdversarialKind, body: FixtureBody<'a>) -> Self {
        Self { kind, body }
    }

    /// Returns the adversarial category.
    #[must_use]
    pub const fn kind(self) -> AdversarialKind {
        self.kind
    }

    /// Returns the compact fixture body.
    #[must_use]
    pub const fn body(self) -> FixtureBody<'a> {
        self.body
    }
}

/// Creates the fixed six-case adversarial response corpus.
pub fn adversarial_corpus() -> Result<[AdversarialFixture<'static>; 6], FixtureBodyError> {
    let oversized_len = DEFAULT_RESPONSE_LIMIT
        .checked_add(1)
        .ok_or(FixtureBodyError::TooLarge)?;
    Ok([
        AdversarialFixture::new(
            AdversarialKind::MalformedJson,
            FixtureBody::new(br#"{"error":"#)?,
        ),
        AdversarialFixture::new(
            AdversarialKind::UnknownFields,
            FixtureBody::new(
                br#"{"action":{"id":1,"command":"test","status":"running","progress":1,"started":"2026-07-12T12:00:00Z","finished":null,"resources":[],"error":null,"unknown":true},"unknown":true}"#,
            )?,
        ),
        AdversarialFixture::new(
            AdversarialKind::MissingRequiredFields,
            FixtureBody::new(br#"{}"#)?,
        ),
        AdversarialFixture::new(
            AdversarialKind::OversizedResponse,
            FixtureBody::repeated(b' ', oversized_len)?,
        ),
        AdversarialFixture::new(
            AdversarialKind::InvalidPagination,
            FixtureBody::new(br#"{"meta":{"pagination":{"page":0,"per_page":0,"last_page":0}}}"#)?,
        ),
        AdversarialFixture::new(
            AdversarialKind::InvalidActionState,
            FixtureBody::new(br#"{"action":{"id":1,"status":"not-a-state","progress":101}}"#)?,
        ),
    ])
}
