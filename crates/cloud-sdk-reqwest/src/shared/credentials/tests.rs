use std::boxed::Box;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicUsize, Ordering},
};

use cloud_sdk::authentication::{CredentialLifetime, CredentialLifetimeState, CredentialTimestamp};
use cloud_sdk_sanitization::SecretBuffer;

use super::{
    BearerCredentialSnapshot, BearerRefreshHandoff, BearerToken, CredentialStore,
    CredentialUpdateError, RefreshHandoffError, TokenRefreshError, TokenRotationError,
};

fn handoff(snapshot: &BearerCredentialSnapshot) -> BearerRefreshHandoff {
    snapshot
        .refresh_handoff()
        .unwrap_or_else(|_| unreachable!())
}

fn lifetime(observed: u64) -> CredentialLifetime {
    CredentialLifetime::from_expires_in(CredentialTimestamp::from_seconds(observed), 3_599, 300)
        .unwrap_or_else(|_| unreachable!())
}

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
        unreachable!("security fixture construction failed");
    };
    let store = CredentialStore::new(active, None);
    let mut rejected = *b"bad token";
    assert!(matches!(
        store.rotate_from_mut_bytes(&mut rejected),
        Err(TokenRotationError::TokenRejected(_))
    ));
    assert_eq!(rejected, [0; 9]);
    let snapshot = store.snapshot();
    assert!(snapshot.is_ok());
    let Ok(snapshot) = snapshot else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(snapshot.owned_bytes(), b"Bearer active-token");
}

#[test]
fn retired_token_waits_for_last_snapshot_and_generations_advance() {
    let drops = Arc::new(AtomicUsize::new(0));
    let active = BearerToken::with_drop_probe("old-token", Arc::clone(&drops));
    let Ok(active) = active else {
        unreachable!("security fixture construction failed")
    };
    let store = CredentialStore::new(active, None);
    let old_snapshot = store.snapshot();
    let Ok(old_snapshot) = old_snapshot else {
        unreachable!("security fixture construction failed");
    };
    let Ok(replacement) = BearerToken::new("new-token") else {
        unreachable!("security fixture construction failed");
    };

    let generation = store.rotate(replacement);
    assert_eq!(generation.map(|value| value.get()), Ok(2));
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert_eq!(old_snapshot.owned_bytes(), b"Bearer old-token");
    let new_snapshot = store.snapshot();
    assert!(new_snapshot.is_ok());
    let Ok(new_snapshot) = new_snapshot else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(new_snapshot.generation().get(), 2);
    assert_eq!(new_snapshot.owned_bytes(), b"Bearer new-token");

    drop(old_snapshot);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn stale_refresh_cannot_overwrite_newer_rotation() {
    let Ok(active) = BearerToken::new("active-token") else {
        unreachable!("security fixture construction failed");
    };
    let store = CredentialStore::new(active, None);
    let Ok(snapshot) = store.snapshot() else {
        unreachable!("security fixture construction failed");
    };
    let handoff = handoff(&snapshot);
    let Ok(rotated) = BearerToken::new("rotated-token") else {
        unreachable!("security fixture construction failed");
    };
    assert!(store.rotate(rotated).is_ok());
    let Ok(stale) = BearerToken::new("stale-refresh") else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(
        store.refresh(handoff, stale),
        Err(TokenRefreshError::StaleGeneration)
    );
    let current = store.snapshot();
    assert!(current.is_ok());
    let Ok(current) = current else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(current.owned_bytes(), b"Bearer rotated-token");
    assert_eq!(current.generation().get(), 2);
}

#[test]
fn refresh_handoff_cannot_cross_credential_store_lineages() {
    let Ok(first_token) = BearerToken::new("first-token") else {
        unreachable!("security fixture construction failed");
    };
    let Ok(second_token) = BearerToken::new("second-token") else {
        unreachable!("security fixture construction failed");
    };
    let first = CredentialStore::new(first_token, None);
    let second = CredentialStore::new(second_token, None);
    let Ok(first_snapshot) = first.snapshot() else {
        unreachable!("security fixture construction failed");
    };
    let drops = Arc::new(AtomicUsize::new(0));
    let Ok(replacement) = BearerToken::with_drop_probe("foreign-replacement", Arc::clone(&drops))
    else {
        unreachable!("security fixture construction failed");
    };

    assert_eq!(
        second.refresh(handoff(&first_snapshot), replacement),
        Err(TokenRefreshError::CredentialMismatch)
    );
    assert_eq!(
        second.snapshot().map(|value| value.generation().get()),
        Ok(1)
    );
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    let current = second.snapshot();
    assert!(current.is_ok());
    let Ok(current) = current else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(current.owned_bytes(), b"Bearer second-token");
}

#[test]
fn competing_refreshes_allow_exactly_one_generation_transition() {
    let Ok(active) = BearerToken::new("active-token") else {
        unreachable!("security fixture construction failed");
    };
    let store = CredentialStore::new(active, None);
    let Ok(snapshot) = store.snapshot() else {
        unreachable!("security fixture construction failed");
    };
    let handoff = handoff(&snapshot);
    let first = BearerToken::new("first-refresh");
    let second = BearerToken::new("second-refresh");
    let (Ok(first), Ok(second)) = (first, second) else {
        unreachable!("security fixture construction failed");
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
        unreachable!("security fixture construction failed");
    };
    let store = Arc::new(CredentialStore::new(active, None));
    let Ok(snapshot) = store.snapshot() else {
        unreachable!("security fixture construction failed");
    };
    let handoff = handoff(&snapshot);
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
        unreachable!("security fixture construction failed");
    };
    let store = CredentialStore::new(active, None);
    let Ok(snapshot) = store.snapshot() else {
        unreachable!("security fixture construction failed");
    };
    let mut rejected = *b"bad token";
    assert!(matches!(
        store.refresh_from_mut_bytes(handoff(&snapshot), &mut rejected),
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
        unreachable!("security fixture construction failed");
    };
    let store = CredentialStore::new(active, None);

    poison_state(&store);
    let snapshot = store.snapshot();
    assert!(snapshot.is_ok());
    assert!(!store.current.is_poisoned());
    let Ok(snapshot) = snapshot else {
        unreachable!("security fixture construction failed")
    };

    poison_state(&store);
    let Ok(replacement) = BearerToken::new("replacement-token") else {
        unreachable!("security fixture construction failed");
    };
    assert!(store.refresh(handoff(&snapshot), replacement).is_ok());
    assert!(!store.current.is_poisoned());
    let snapshot = store.snapshot();
    assert!(snapshot.is_ok());
    let Ok(snapshot) = snapshot else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(snapshot.owned_bytes(), b"Bearer replacement-token");
}

#[test]
fn header_copy_has_cleanup_owner_and_redacted_snapshot() {
    let drops = Arc::new(AtomicUsize::new(0));
    let token = BearerToken::with_header_drop_probe("active-token", Arc::clone(&drops));
    let Ok(token) = token else {
        unreachable!("security fixture construction failed")
    };
    let store = CredentialStore::new(token, None);
    let Ok(snapshot) = store.snapshot() else {
        unreachable!("security fixture construction failed");
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

#[test]
fn expiring_snapshots_require_time_qualified_refresh_handoffs() {
    let Ok(token) = BearerToken::new("expiring-token") else {
        unreachable!("security fixture construction failed");
    };
    let store = CredentialStore::new(token, Some(lifetime(1_000)));
    let Ok(snapshot) = store.snapshot() else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(snapshot.lifetime(), Some(lifetime(1_000)));
    assert!(matches!(
        snapshot.refresh_handoff(),
        Err(RefreshHandoffError::ExplicitTimeRequired)
    ));
    assert!(matches!(
        snapshot.refresh_handoff_at(CredentialTimestamp::from_seconds(999)),
        Err(RefreshHandoffError::ClockRollback)
    ));
    assert!(matches!(
        snapshot.refresh_handoff_at(CredentialTimestamp::from_seconds(4_298)),
        Err(RefreshHandoffError::RefreshNotRequired)
    ));
    assert!(
        snapshot
            .refresh_handoff_at(CredentialTimestamp::from_seconds(4_299))
            .is_ok()
    );
    assert!(matches!(
        snapshot.refresh_handoff_at(CredentialTimestamp::from_seconds(4_599)),
        Err(RefreshHandoffError::CredentialExpired)
    ));
}

#[test]
fn lifetime_mode_cannot_be_silently_added_or_removed() {
    let Ok(expiring) = BearerToken::new("expiring-token") else {
        unreachable!("security fixture construction failed");
    };
    let expiring = CredentialStore::new(expiring, Some(lifetime(1_000)));
    let Ok(replacement) = BearerToken::new("replacement-token") else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(
        expiring.rotate(replacement),
        Err(CredentialUpdateError::LifetimeRequired)
    );
    assert_eq!(
        expiring.snapshot().map(|snapshot| snapshot
            .lifetime()
            .map(|value| value.state_at(CredentialTimestamp::from_seconds(1_000)))),
        Ok(Some(CredentialLifetimeState::Fresh))
    );

    let Ok(static_token) = BearerToken::new("static-token") else {
        unreachable!("security fixture construction failed");
    };
    let static_store = CredentialStore::new(static_token, None);
    let Ok(replacement) = BearerToken::new("replacement-token") else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(
        static_store.rotate_with_lifetime(replacement, lifetime(2_000)),
        Err(CredentialUpdateError::LifetimeForbidden)
    );
}

#[test]
fn expiring_refresh_rotates_token_and_lifetime_atomically_and_clears_input() {
    let Ok(active) = BearerToken::new("active-token") else {
        unreachable!("security fixture construction failed");
    };
    let store = CredentialStore::new(active, Some(lifetime(1_000)));
    let Ok(snapshot) = store.snapshot() else {
        unreachable!("security fixture construction failed");
    };
    let handoff = snapshot
        .refresh_handoff_at(CredentialTimestamp::from_seconds(4_299))
        .unwrap_or_else(|_| unreachable!());
    let mut replacement = *b"refreshed-token";
    assert_eq!(
        store
            .refresh_from_mut_bytes_with_lifetime(
                handoff.clone(),
                &mut replacement,
                lifetime(4_300),
            )
            .map(|generation| generation.get()),
        Ok(2)
    );
    assert_eq!(replacement, [0; 15]);
    let Ok(current) = store.snapshot() else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(current.owned_bytes(), b"Bearer refreshed-token");
    assert_eq!(current.lifetime(), Some(lifetime(4_300)));
    assert_eq!(
        store.refresh_with_lifetime(
            handoff,
            BearerToken::new("stale-token").unwrap_or_else(|_| unreachable!()),
            lifetime(4_301),
        ),
        Err(TokenRefreshError::StaleGeneration)
    );
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
