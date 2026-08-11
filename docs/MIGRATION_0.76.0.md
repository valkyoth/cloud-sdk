# Migrating Source Users To v0.76.0

v0.76.0 is an internal source milestone. The latest crates.io checkpoint is
v0.75.0, and cumulative publication is deferred to v0.80.0.

## Provider-Neutral Attempt State

Custom authentication layers can use the new allocation-free state machine:

```rust
use cloud_sdk::authentication::{
    CredentialReconfirmation, SharedCredentialAttemptState,
};

let state = SharedCredentialAttemptState::new();
let attempt = state.begin()?;

// Report this only after the provider classifies authentication rejection.
state.reject(attempt)?;
assert!(state.begin().is_err());

// This must be an explicit caller decision, never automatic retry policy.
state.reconfirm(
    attempt.generation(),
    CredentialReconfirmation::acknowledge_same_credentials(),
)?;
# Ok::<(), Box<dyn core::error::Error>>(())
```

Replacement credentials use `replace(expected_generation)` instead. Stale
attempts cannot close or reopen the replacement generation. An attempt now
borrows the exact `SharedCredentialAttemptState` that issued it; passing it to
another state returns `CredentialAttemptError::ForeignState`, including when
both states have the same generation number.

## Protected Robot Credentials

Source users can enable `cloud-sdk-hetzner/alloc` and ingest clearable input:

```rust
use cloud_sdk_hetzner::robot::RobotCredentials;

let mut username = b"example-user".to_vec();
let mut password = b"example-only-secret".to_vec();
let credentials = RobotCredentials::from_mut_bytes(&mut username, &mut password)?;
assert!(username.iter().all(|byte| *byte == 0));
assert!(password.iter().all(|byte| *byte == 0));

let attempt = credentials.begin_attempt()?;
credentials.try_with_attempt(&attempt, |username, password| {
    // Pass borrowed values directly into a reviewed Basic encoder.
    assert!(!username.is_empty() && !password.is_empty());
})?;
# Ok::<(), Box<dyn core::error::Error>>(())
```

`RobotCredentialAttempt` owns an opaque shared lineage rather than borrowing
`RobotCredentials`. It can move into an owned task, and mutable credential
rotation may proceed while an older request is in flight. Classifying that old
response afterward returns `StaleGeneration`. Beginning an attempt clones the
lineage without a per-attempt allocation.

`RobotCredentials` does not send requests or classify responses. Do not build
an automatic 401 retry. Live testing must never intentionally use invalid
Robot credentials.

The provider's `alloc` feature now explicitly activates
`cloud-sdk-sanitization/alloc`; `cloud-sdk-hetzner` production library builds
with either `--features alloc` or `--features std` no longer depend on test
feature unification.

## Published Dependencies

Crates.io users remain on the v0.75 checkpoint until v0.80:

```toml
[dependencies]
cloud-sdk = "0.75.0"
cloud-sdk-hetzner = "0.42.0"
```
