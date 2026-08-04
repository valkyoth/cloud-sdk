//! Validated finite pagination and action-polling scenario scripts.

use crate::{
    ActionState, DynamicRequest, FixtureKind, MAX_DYNAMIC_RECORDS, ProviderFixtureBuilder,
    ResponseFixture,
};

/// Invalid or exhausted deterministic scenario script.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScenarioScriptError {
    /// A script requires at least one response fixture.
    Empty,
    /// The script exceeds the bounded dynamic-record limit.
    TooManySteps,
    /// A pagination script contains a non-pagination fixture.
    ExpectedPagination,
    /// Pagination pages must begin at one and increase by one.
    InvalidPageSequence,
    /// Page size, entry total, and last page must remain stable.
    PaginationMetadataChanged,
    /// The final pagination fixture must represent the last page.
    PaginationDidNotFinish,
    /// An action script contains a non-action fixture.
    ExpectedAction,
    /// Action progress must not decrease.
    ActionProgressDecreased,
    /// Only the final action fixture may be terminal.
    ActionFinishedEarly,
    /// The final action fixture must be terminal.
    ActionDidNotFinish,
    /// No scripted response remains.
    Exhausted,
}

impl_static_error!(ScenarioScriptError,
    Self::Empty => "scenario script is empty",
    Self::TooManySteps => "scenario script exceeds the step limit",
    Self::ExpectedPagination => "pagination script contains a different fixture kind",
    Self::InvalidPageSequence => "pagination script page sequence is invalid",
    Self::PaginationMetadataChanged => "pagination script metadata changes between pages",
    Self::PaginationDidNotFinish => "pagination script does not finish on the last page",
    Self::ExpectedAction => "action script contains a different fixture kind",
    Self::ActionProgressDecreased => "action script progress decreases",
    Self::ActionFinishedEarly => "action script finishes before its final step",
    Self::ActionDidNotFinish => "action script does not finish",
    Self::Exhausted => "scenario script has no response remaining",
);

/// Coherent finite pagination response sequence.
pub struct PaginationScript<'fixture> {
    fixtures: &'fixture [ResponseFixture<'fixture>],
}

impl<'fixture> PaginationScript<'fixture> {
    /// Validates a complete page-one-through-last-page sequence.
    pub fn new(
        fixtures: &'fixture [ResponseFixture<'fixture>],
    ) -> Result<Self, ScenarioScriptError> {
        validate_length(fixtures)?;
        let first = fixtures
            .first()
            .and_then(ResponseFixture::pagination)
            .ok_or(ScenarioScriptError::ExpectedPagination)?;
        let mut expected_page = 1_u64;
        for fixture in fixtures {
            if fixture.kind() != FixtureKind::Pagination {
                return Err(ScenarioScriptError::ExpectedPagination);
            }
            let page = fixture
                .pagination()
                .ok_or(ScenarioScriptError::ExpectedPagination)?;
            if page.page() != expected_page {
                return Err(ScenarioScriptError::InvalidPageSequence);
            }
            if page.per_page() != first.per_page()
                || page.total_entries() != first.total_entries()
                || page.last_page() != first.last_page()
            {
                return Err(ScenarioScriptError::PaginationMetadataChanged);
            }
            expected_page = expected_page
                .checked_add(1)
                .ok_or(ScenarioScriptError::InvalidPageSequence)?;
        }
        let last = fixtures
            .last()
            .and_then(ResponseFixture::pagination)
            .ok_or(ScenarioScriptError::ExpectedPagination)?;
        if last.page() != last.last_page() {
            return Err(ScenarioScriptError::PaginationDidNotFinish);
        }
        Ok(Self { fixtures })
    }

    /// Returns the validated number of page responses.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.fixtures.len()
    }

    /// Reports whether this script has no steps. Valid scripts are never empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.fixtures.is_empty()
    }
}

impl<'fixture> ProviderFixtureBuilder<'fixture> for PaginationScript<'fixture> {
    type Error = ScenarioScriptError;

    fn build<'request>(
        &self,
        request: DynamicRequest<'request>,
    ) -> Result<&'fixture ResponseFixture<'fixture>, Self::Error> {
        self.fixtures
            .get(request.sequence())
            .ok_or(ScenarioScriptError::Exhausted)
    }
}

/// Coherent finite running-to-terminal action response sequence.
pub struct ActionScript<'fixture> {
    fixtures: &'fixture [ResponseFixture<'fixture>],
}

impl<'fixture> ActionScript<'fixture> {
    /// Validates nondecreasing progress and a single final terminal state.
    pub fn new(
        fixtures: &'fixture [ResponseFixture<'fixture>],
    ) -> Result<Self, ScenarioScriptError> {
        validate_length(fixtures)?;
        let mut previous_progress = 0_u8;
        let final_index = fixtures
            .len()
            .checked_sub(1)
            .ok_or(ScenarioScriptError::Empty)?;
        for (index, fixture) in fixtures.iter().enumerate() {
            if fixture.kind() != FixtureKind::Action {
                return Err(ScenarioScriptError::ExpectedAction);
            }
            let action = fixture
                .action_metadata()
                .ok_or(ScenarioScriptError::ExpectedAction)?;
            if action.progress() < previous_progress {
                return Err(ScenarioScriptError::ActionProgressDecreased);
            }
            let terminal = !matches!(action.state(), ActionState::Running);
            if terminal && index != final_index {
                return Err(ScenarioScriptError::ActionFinishedEarly);
            }
            if !terminal && index == final_index {
                return Err(ScenarioScriptError::ActionDidNotFinish);
            }
            previous_progress = action.progress();
        }
        Ok(Self { fixtures })
    }

    /// Returns the validated number of polling responses.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.fixtures.len()
    }

    /// Reports whether this script has no steps. Valid scripts are never empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.fixtures.is_empty()
    }
}

impl<'fixture> ProviderFixtureBuilder<'fixture> for ActionScript<'fixture> {
    type Error = ScenarioScriptError;

    fn build<'request>(
        &self,
        request: DynamicRequest<'request>,
    ) -> Result<&'fixture ResponseFixture<'fixture>, Self::Error> {
        self.fixtures
            .get(request.sequence())
            .ok_or(ScenarioScriptError::Exhausted)
    }
}

fn validate_length(fixtures: &[ResponseFixture<'_>]) -> Result<(), ScenarioScriptError> {
    if fixtures.is_empty() {
        return Err(ScenarioScriptError::Empty);
    }
    if fixtures.len() > MAX_DYNAMIC_RECORDS {
        return Err(ScenarioScriptError::TooManySteps);
    }
    Ok(())
}
