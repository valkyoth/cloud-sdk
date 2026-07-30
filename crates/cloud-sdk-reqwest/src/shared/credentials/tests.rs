use std::boxed::Box;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicUsize, Ordering},
};

use cloud_sdk_sanitization::SecretBuffer;

use super::{BearerToken, CredentialStore, TokenRefreshError, TokenRotationError};

#[test]
fn mutable_and_guarded_sources_clear_on_success_and_failure() {
    let mut valid = *b"replacement";
    let token = BearerToken::from_mut_bytes(&mut valid);
    assert!(token.is_ok());
    assert_eq!(valid, [0; 11]);

    let mut invalid = *b"bad token";
    assert!(BearerToken::from_mut_bytes(&mut invalid).is_err());
    assert_eq!(invalid, [0; 9]);

    let mut guarded = *b"guarded-token";
    let token = BearerToken::from_secret_buffer(SecretBuffer::new(&mut guarded));
    assert!(token.is_ok());
    assert_eq!(guarded, [0; 13]);
}

#[test]
fn rejected_rotation_preserves_active_token_and_clears_input() {
    let Ok(active) = BearerToken::new("active-token") else {
        return;
    };
    let store = CredentialStore::new(active);
    let mut rejected = *b"bad token";
    assert!(matches!(
        store.rotate_from_mut_bytes(&mut rejected),
        Err(TokenRotationError::TokenRejected(_))
    ));
    assert_eq!(rejected, [0; 9]);
    let snapshot = store.snapshot();
    assert!(snapshot.is_ok());
    if let Ok(snapshot) = snapshot {
        assert_eq!(snapshot.owned_bytes(), b"Bearer active-token");
    }
}

#[test]
fn retired_token_waits_for_last_snapshot_and_generations_advance() {
    let drops = Arc::new(AtomicUsize::new(0));
    let active = BearerToken::with_drop_probe("old-token", Arc::clone(&drops));
    let Ok(active) = active else { return };
    let store = CredentialStore::new(active);
    let old_snapshot = store.snapshot();
    let Ok(old_snapshot) = old_snapshot else {
        return;
    };
    let Ok(replacement) = BearerToken::new("new-token") else {
        return;
    };

    let generation = store.rotate(replacement);
    assert_eq!(generation.map(|value| value.get()), Ok(2));
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert_eq!(old_snapshot.owned_bytes(), b"Bearer old-token");
    let new_snapshot = store.snapshot();
    assert!(new_snapshot.is_ok());
    if let Ok(new_snapshot) = new_snapshot {
        assert_eq!(new_snapshot.generation().get(), 2);
        assert_eq!(new_snapshot.owned_bytes(), b"Bearer new-token");
    }
    drop(old_snapshot);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn stale_refresh_cannot_overwrite_newer_rotation() {
    let Ok(active) = BearerToken::new("active-token") else {
        return;
    };
    let store = CredentialStore::new(active);
    let Ok(snapshot) = store.snapshot() else {
        return;
    };
    let handoff = snapshot.refresh_handoff();
    let Ok(rotated) = BearerToken::new("rotated-token") else {
        return;
    };
    assert!(store.rotate(rotated).is_ok());
    let Ok(stale) = BearerToken::new("stale-refresh") else {
        return;
    };
    assert_eq!(
        store.refresh(handoff, stale),
        Err(TokenRefreshError::StaleGeneration)
    );
    let current = store.snapshot();
    assert!(current.is_ok());
    if let Ok(current) = current {
        assert_eq!(current.owned_bytes(), b"Bearer rotated-token");
        assert_eq!(current.generation().get(), 2);
    }
}

#[test]
fn refresh_handoff_cannot_cross_credential_store_lineages() {
    let Ok(first_token) = BearerToken::new("first-token") else {
        return;
    };
    let Ok(second_token) = BearerToken::new("second-token") else {
        return;
    };
    let first = CredentialStore::new(first_token);
    let second = CredentialStore::new(second_token);
    let Ok(first_snapshot) = first.snapshot() else {
        return;
    };
    let drops = Arc::new(AtomicUsize::new(0));
    let Ok(replacement) = BearerToken::with_drop_probe("foreign-replacement", Arc::clone(&drops))
    else {
        return;
    };

    assert_eq!(
        second.refresh(first_snapshot.refresh_handoff(), replacement),
        Err(TokenRefreshError::CredentialMismatch)
    );
    assert_eq!(
        second.snapshot().map(|value| value.generation().get()),
        Ok(1)
    );
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    let current = second.snapshot();
    assert!(current.is_ok());
    if let Ok(current) = current {
        assert_eq!(current.owned_bytes(), b"Bearer second-token");
    }
}

#[test]
fn competing_refreshes_allow_exactly_one_generation_transition() {
    let Ok(active) = BearerToken::new("active-token") else {
        return;
    };
    let store = CredentialStore::new(active);
    let Ok(snapshot) = store.snapshot() else {
        return;
    };
    let handoff = snapshot.refresh_handoff();
    let first = BearerToken::new("first-refresh");
    let second = BearerToken::new("second-refresh");
    let (Ok(first), Ok(second)) = (first, second) else {
        return;
    };
    assert_eq!(
        store
            .refresh(handoff.clone(), first)
            .map(|value| value.get()),
        Ok(2)
    );
    assert_eq!(
        store.refresh(handoff, second),
        Err(TokenRefreshError::StaleGeneration)
    );
}

#[test]
fn concurrent_refresh_race_allows_exactly_one_winner() {
    let Ok(active) = BearerToken::new("active-token") else {
        return;
    };
    let store = Arc::new(CredentialStore::new(active));
    let Ok(snapshot) = store.snapshot() else {
        return;
    };
    let handoff = snapshot.refresh_handoff();
    let barrier = Arc::new(Barrier::new(3));
    let mut successes = 0_usize;
    let mut stale = 0_usize;
    std::thread::scope(|scope| {
        let first_store = Arc::clone(&store);
        let first_barrier = Arc::clone(&barrier);
        let first_handoff = handoff.clone();
        let first = scope.spawn(move || {
            first_barrier.wait();
            match BearerToken::new("first-refresh") {
                Ok(token) => first_store.refresh(first_handoff, token),
                Err(error) => Err(TokenRefreshError::TokenRejected(error)),
            }
        });
        let second_store = Arc::clone(&store);
        let second_barrier = Arc::clone(&barrier);
        let second = scope.spawn(move || {
            second_barrier.wait();
            let token = BearerToken::new("second-refresh");
            match token {
                Ok(token) => second_store.refresh(handoff, token),
                Err(error) => Err(TokenRefreshError::TokenRejected(error)),
            }
        });
        barrier.wait();
        for result in [first.join(), second.join()] {
            match result {
                Ok(Ok(_)) => successes = successes.saturating_add(1),
                Ok(Err(_)) => stale = stale.saturating_add(1),
                Err(_) => {}
            }
        }
    });
    assert_eq!(successes, 1);
    assert_eq!(stale, 1);
    assert_eq!(
        store.snapshot().map(|value| value.generation().get()),
        Ok(2)
    );
}

#[test]
fn rejected_refresh_clears_input_without_changing_generation() {
    let Ok(active) = BearerToken::new("active-token") else {
        return;
    };
    let store = CredentialStore::new(active);
    let Ok(snapshot) = store.snapshot() else {
        return;
    };
    let mut rejected = *b"bad token";
    assert!(matches!(
        store.refresh_from_mut_bytes(snapshot.refresh_handoff(), &mut rejected),
        Err(TokenRefreshError::TokenRejected(_))
    ));
    assert_eq!(rejected, [0; 9]);
    assert_eq!(
        store.snapshot().map(|value| value.generation().get()),
        Ok(1)
    );
}

#[test]
fn poisoned_state_recovers_for_snapshots_rotations_and_refreshes() {
    let Ok(active) = BearerToken::new("active-token") else {
        return;
    };
    let store = CredentialStore::new(active);

    poison_state(&store);
    let snapshot = store.snapshot();
    assert!(snapshot.is_ok());
    assert!(!store.current.is_poisoned());
    let Ok(snapshot) = snapshot else { return };

    poison_state(&store);
    let Ok(replacement) = BearerToken::new("replacement-token") else {
        return;
    };
    assert!(
        store
            .refresh(snapshot.refresh_handoff(), replacement)
            .is_ok()
    );
    assert!(!store.current.is_poisoned());
    let snapshot = store.snapshot();
    assert!(snapshot.is_ok());
    if let Ok(snapshot) = snapshot {
        assert_eq!(snapshot.owned_bytes(), b"Bearer replacement-token");
    }
}

#[test]
fn header_copy_has_cleanup_owner_and_redacted_snapshot() {
    let drops = Arc::new(AtomicUsize::new(0));
    let token = BearerToken::with_header_drop_probe("active-token", Arc::clone(&drops));
    let Ok(token) = token else { return };
    let store = CredentialStore::new(token);
    let Ok(snapshot) = store.snapshot() else {
        return;
    };
    let header = snapshot.header_value_with_drop_probe();
    assert!(header.is_ok());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(header);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    let debug = std::format!("{snapshot:?}");
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("active-token"));
}

fn poison_state(store: &CredentialStore) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let guard = store.current.write();
        let Ok(_guard) = guard else { return };
        resume_unwind(Box::new(()));
    }));
    assert!(result.is_err());
    assert!(store.current.is_poisoned());
}
