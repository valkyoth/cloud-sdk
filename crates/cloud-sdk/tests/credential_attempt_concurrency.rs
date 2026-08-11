//! Cross-thread credential-attempt lifecycle evidence.

use std::sync::{Arc, Barrier};
use std::thread;

use cloud_sdk::authentication::{
    CredentialAttemptError, CredentialAttemptGeneration, SharedCredentialAttemptState,
};

#[test]
fn concurrent_attempts_share_one_generation_and_rejection_is_global() {
    let state = Arc::new(SharedCredentialAttemptState::new());
    let started = Arc::new(Barrier::new(8));
    let rejected = Arc::new(Barrier::new(8));
    let mut workers = Vec::new();
    for worker_index in 0..8 {
        let state = Arc::clone(&state);
        let started = Arc::clone(&started);
        let rejected = Arc::clone(&rejected);
        workers.push(thread::spawn(move || {
            let attempt = state
                .begin()
                .unwrap_or_else(|_| unreachable!("shared generation closed too early"));
            let generation = attempt.generation();
            started.wait();
            if worker_index == 0 {
                assert_eq!(state.reject(attempt), Ok(()));
            }
            rejected.wait();
            (generation, state.validate(attempt))
        }));
    }

    let outcomes = workers
        .into_iter()
        .map(|worker| {
            worker
                .join()
                .unwrap_or_else(|_| unreachable!("credential worker panicked"))
        })
        .collect::<Vec<_>>();
    assert!(outcomes.iter().all(|(generation, result)| {
        *generation == CredentialAttemptGeneration::INITIAL
            && *result == Err(CredentialAttemptError::GenerationRejected)
    }));
    assert_eq!(
        state.begin(),
        Err(CredentialAttemptError::GenerationRejected)
    );
}
