//! Cross-thread credential-attempt lifecycle evidence.

use std::sync::{Arc, Barrier};
use std::thread;

use cloud_sdk::authentication::{
    CredentialAttemptError, CredentialAttemptGeneration, SharedCredentialAttemptState,
};

#[test]
fn concurrent_attempts_share_one_generation_and_rejection_is_global() {
    let state = Arc::new(SharedCredentialAttemptState::new());
    let barrier = Arc::new(Barrier::new(9));
    let mut workers = Vec::new();
    for _ in 0..8 {
        let state = Arc::clone(&state);
        let barrier = Arc::clone(&barrier);
        workers.push(thread::spawn(move || {
            let attempt = state
                .begin()
                .unwrap_or_else(|_| unreachable!("shared generation closed too early"));
            barrier.wait();
            attempt
        }));
    }
    barrier.wait();

    let attempts = workers
        .into_iter()
        .map(|worker| {
            worker
                .join()
                .unwrap_or_else(|_| unreachable!("credential worker panicked"))
        })
        .collect::<Vec<_>>();
    assert!(
        attempts
            .iter()
            .all(|attempt| attempt.generation() == CredentialAttemptGeneration::INITIAL)
    );

    let first = attempts
        .first()
        .copied()
        .unwrap_or_else(|| unreachable!("credential attempt fixture is empty"));
    assert_eq!(state.reject(first), Ok(()));
    assert_eq!(
        state.begin(),
        Err(CredentialAttemptError::GenerationRejected)
    );
    for attempt in attempts {
        assert_eq!(state.reject(attempt), Ok(()));
    }
}
