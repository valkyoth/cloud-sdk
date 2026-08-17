use super::{
    CredentialAttemptError, CredentialAttemptGeneration, CredentialAttemptStatus,
    CredentialReconfirmation, MAX_GENERATION, SharedCredentialAttemptState,
};

#[test]
fn rejection_closes_one_generation_until_replaced_or_reconfirmed() {
    let state = SharedCredentialAttemptState::new();
    let first = state
        .begin()
        .unwrap_or_else(|_| unreachable!("initial credential generation was closed"));
    assert_eq!(first.generation(), CredentialAttemptGeneration::INITIAL);
    assert_eq!(
        state.reconfirm(
            first.generation(),
            CredentialReconfirmation::acknowledge_same_credentials(),
        ),
        Err(CredentialAttemptError::ReconfirmationNotRequired)
    );
    assert_eq!(state.reject(first), Ok(()));
    assert_eq!(
        state.validate(first),
        Err(CredentialAttemptError::GenerationRejected)
    );
    assert_eq!(state.reject(first), Ok(()));
    assert_eq!(
        state.begin(),
        Err(CredentialAttemptError::GenerationRejected)
    );

    let second = state
        .reconfirm(
            first.generation(),
            CredentialReconfirmation::acknowledge_same_credentials(),
        )
        .unwrap_or_else(|_| unreachable!("explicit reconfirmation was rejected"));
    assert_eq!(second.get(), 2);
    assert!(state.begin().is_ok());

    let third = state
        .replace(second)
        .unwrap_or_else(|_| unreachable!("replacement generation was rejected"));
    assert_eq!(third.get(), 3);
    assert!(state.begin().is_ok());
}

#[test]
fn one_dispatch_is_exclusive_until_classification_finishes() {
    let state = SharedCredentialAttemptState::new();
    let first = state
        .begin()
        .unwrap_or_else(|_| unreachable!("initial credential generation was closed"));
    let second = state
        .begin()
        .unwrap_or_else(|_| unreachable!("second credential attempt was closed"));
    let guard = state
        .reserve_dispatch(first)
        .unwrap_or_else(|_| unreachable!("first dispatch was not admitted"));

    assert_eq!(
        state.reserve_dispatch(second).map(|_| ()),
        Err(CredentialAttemptError::DispatchBusy)
    );
    assert_eq!(
        state.replace(first.generation()),
        Err(CredentialAttemptError::DispatchBusy)
    );
    assert_eq!(guard.reject(), Ok(()));
    drop(guard);
    assert_eq!(
        state.reserve_dispatch(second).map(|_| ()),
        Err(CredentialAttemptError::GenerationRejected)
    );
}

#[test]
fn classified_completion_reopens_but_unclassified_drop_rejects() {
    let state = SharedCredentialAttemptState::new();
    let first = state
        .begin()
        .unwrap_or_else(|_| unreachable!("initial credential generation was closed"));
    state
        .reserve_dispatch(first)
        .unwrap_or_else(|_| unreachable!("classified dispatch was not admitted"))
        .complete();
    let cancelled = state
        .begin()
        .unwrap_or_else(|_| unreachable!("completed dispatch left generation closed"));
    let guard = state
        .reserve_dispatch(cancelled)
        .unwrap_or_else(|_| unreachable!("cancelled dispatch was not admitted"));
    drop(guard);

    assert_eq!(
        state.begin(),
        Err(CredentialAttemptError::GenerationRejected)
    );
}

#[test]
fn stale_transitions_cannot_close_or_reopen_replacement_credentials() {
    let state = SharedCredentialAttemptState::new();
    let stale = state
        .begin()
        .unwrap_or_else(|_| unreachable!("initial credential generation was closed"));
    let current = state
        .replace(stale.generation())
        .unwrap_or_else(|_| unreachable!("replacement generation was rejected"));
    assert_eq!(
        state.reject(stale),
        Err(CredentialAttemptError::StaleGeneration)
    );
    assert_eq!(
        state.validate(stale),
        Err(CredentialAttemptError::StaleGeneration)
    );
    assert_eq!(
        state.replace(stale.generation()),
        Err(CredentialAttemptError::StaleGeneration)
    );
    assert_eq!(
        state.reconfirm(
            stale.generation(),
            CredentialReconfirmation::acknowledge_same_credentials(),
        ),
        Err(CredentialAttemptError::StaleGeneration)
    );
    assert_eq!(state.observe(), (current, CredentialAttemptStatus::Open));
}

#[test]
fn foreign_attempts_never_validate_or_close_equal_generations() {
    let owner_a = SharedCredentialAttemptState::new();
    let owner_b = SharedCredentialAttemptState::new();
    let foreign = owner_a
        .begin()
        .unwrap_or_else(|_| unreachable!("owner A generation was closed"));

    assert_eq!(
        owner_b.validate(foreign),
        Err(CredentialAttemptError::ForeignState)
    );
    assert_eq!(
        owner_b.reject(foreign),
        Err(CredentialAttemptError::ForeignState)
    );
    assert_eq!(
        owner_b.observe(),
        (
            CredentialAttemptGeneration::INITIAL,
            CredentialAttemptStatus::Open
        )
    );

    let generation_a = owner_a
        .replace(CredentialAttemptGeneration::INITIAL)
        .unwrap_or_else(|_| unreachable!("owner A replacement failed"));
    let generation_b = owner_b
        .replace(CredentialAttemptGeneration::INITIAL)
        .unwrap_or_else(|_| unreachable!("owner B replacement failed"));
    assert_eq!(generation_a, generation_b);
    let foreign_replacement = owner_a
        .begin()
        .unwrap_or_else(|_| unreachable!("owner A replacement was closed"));
    assert_eq!(
        owner_b.reject(foreign_replacement),
        Err(CredentialAttemptError::ForeignState)
    );
    assert_eq!(
        owner_b.observe(),
        (generation_b, CredentialAttemptStatus::Open)
    );
}

#[test]
fn generation_exhaustion_fails_closed_without_wrapping() {
    let mut state = SharedCredentialAttemptState::new();
    state.set_generation_for_test(MAX_GENERATION, true);
    let generation = state.observe().0;
    assert_eq!(
        state.replace(generation),
        Err(CredentialAttemptError::GenerationExhausted)
    );
    assert_eq!(
        state.reconfirm(
            generation,
            CredentialReconfirmation::acknowledge_same_credentials(),
        ),
        Err(CredentialAttemptError::GenerationExhausted)
    );
    assert_eq!(
        state.observe(),
        (generation, CredentialAttemptStatus::Rejected)
    );
}
