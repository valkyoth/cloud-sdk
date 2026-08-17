//! Cross-thread credential-attempt lifecycle evidence.

use std::sync::{Arc, Barrier};
use std::thread;

#[cfg(feature = "alloc")]
use cloud_sdk::authentication::OwnedCredentialAttemptState;
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

#[cfg(feature = "alloc")]
#[test]
fn concurrent_owned_dispatch_cannot_pass_admission_twice() {
    let state = Arc::new(OwnedCredentialAttemptState::new());
    let held = state
        .begin()
        .unwrap_or_else(|_| unreachable!("initial owned attempt was rejected"));
    let competing = state
        .begin()
        .unwrap_or_else(|_| unreachable!("competing owned attempt was rejected"));
    let admitted = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let worker_state = Arc::clone(&state);
    let worker_admitted = Arc::clone(&admitted);
    let worker_release = Arc::clone(&release);
    let worker = thread::spawn(move || {
        let guard = worker_state
            .reserve_dispatch(&held)
            .unwrap_or_else(|_| unreachable!("first dispatch was not admitted"));
        worker_admitted.wait();
        worker_release.wait();
        guard
            .reject()
            .unwrap_or_else(|_| unreachable!("guarded rejection failed"));
    });

    admitted.wait();
    assert_eq!(
        state.reserve_dispatch(&competing).map(|_| ()),
        Err(CredentialAttemptError::DispatchBusy)
    );
    release.wait();
    worker
        .join()
        .unwrap_or_else(|_| unreachable!("dispatch worker panicked"));
    assert_eq!(
        state.reserve_dispatch(&competing).map(|_| ()),
        Err(CredentialAttemptError::GenerationRejected)
    );
}
