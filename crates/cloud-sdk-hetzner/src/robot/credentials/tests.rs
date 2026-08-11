use alloc::format;
use alloc::vec;
use alloc::vec::Vec;

use cloud_sdk::authentication::{
    CredentialAttemptError, CredentialAttemptGeneration, CredentialAttemptStatus,
    CredentialReconfirmation, ScopeRequirement, ScopeViolation,
};
use cloud_sdk_sanitization::SecretBuffer;

use super::{
    MAX_ROBOT_PASSWORD_BYTES, MAX_ROBOT_USERNAME_BYTES, RobotCredentialAttempt,
    RobotCredentialError, RobotCredentialRotationError, RobotCredentialScope,
    RobotCredentialStateError, RobotCredentials,
};
use crate::identity::{CLOUD_SERVICE_ID, HETZNER_PROVIDER_ID, ROBOT_SERVICE_ID};

fn credentials() -> RobotCredentials {
    let mut username = Vec::from(b"robot-user".as_slice());
    let mut password = Vec::from(b"replace-only-secret".as_slice());
    RobotCredentials::from_mut_bytes(&mut username, &mut password)
        .unwrap_or_else(|_| unreachable!("Robot credential fixture was rejected"))
}

#[test]
fn mutable_and_guarded_sources_clear_on_success_and_rejection() {
    let mut username = Vec::from(b"robot-user".as_slice());
    let mut password = Vec::from(b"replace-only-secret".as_slice());
    assert!(RobotCredentials::from_mut_bytes(&mut username, &mut password).is_ok());
    assert!(username.iter().all(|byte| *byte == 0));
    assert!(password.iter().all(|byte| *byte == 0));

    let mut invalid_username = Vec::from(b"bad:user".as_slice());
    let mut invalid_password = Vec::from(b"secret".as_slice());
    assert!(matches!(
        RobotCredentials::from_secret_buffers(
            SecretBuffer::new(&mut invalid_username),
            SecretBuffer::new(&mut invalid_password),
        ),
        Err(RobotCredentialError::InvalidUsername)
    ));
    assert!(invalid_username.iter().all(|byte| *byte == 0));
    assert!(invalid_password.iter().all(|byte| *byte == 0));
}

#[test]
fn exact_component_bounds_and_ascii_profiles_are_enforced() {
    let mut username = vec![b'u'; MAX_ROBOT_USERNAME_BYTES];
    let mut password = vec![b'p'; MAX_ROBOT_PASSWORD_BYTES];
    assert!(RobotCredentials::from_mut_bytes(&mut username, &mut password).is_ok());

    for (mut username, mut password, expected) in [
        (
            Vec::new(),
            Vec::from(b"p".as_slice()),
            RobotCredentialError::EmptyUsername,
        ),
        (
            vec![b'u'; MAX_ROBOT_USERNAME_BYTES + 1],
            Vec::from(b"p".as_slice()),
            RobotCredentialError::UsernameTooLong,
        ),
        (
            Vec::from(b"user name".as_slice()),
            Vec::from(b"p".as_slice()),
            RobotCredentialError::InvalidUsername,
        ),
        (
            Vec::from(b"user".as_slice()),
            Vec::new(),
            RobotCredentialError::EmptyPassword,
        ),
        (
            Vec::from(b"user".as_slice()),
            vec![b'p'; MAX_ROBOT_PASSWORD_BYTES + 1],
            RobotCredentialError::PasswordTooLong,
        ),
        (
            Vec::from(b"user".as_slice()),
            vec![0],
            RobotCredentialError::InvalidPassword,
        ),
    ] {
        assert!(matches!(
            RobotCredentials::from_mut_bytes(&mut username, &mut password),
            Err(error) if error == expected
        ));
        assert!(username.iter().all(|byte| *byte == 0));
        assert!(password.iter().all(|byte| *byte == 0));
    }
}

#[test]
fn rejection_closes_secret_access_until_rotation_or_explicit_reconfirmation() {
    let mut credentials = credentials();
    let first = credentials
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("initial Robot attempt was rejected"));
    assert_eq!(
        credentials.try_with_attempt(&first, |username, password| {
            username == "robot-user" && password == "replace-only-secret"
        }),
        Ok(true)
    );
    assert_eq!(credentials.reject_attempt(&first), Ok(()));
    assert_eq!(
        credentials.begin_attempt(),
        Err(RobotCredentialStateError::Attempt(
            CredentialAttemptError::GenerationRejected
        ))
    );
    assert_eq!(
        credentials.try_with_attempt(&first, |_, _| ()),
        Err(RobotCredentialStateError::Attempt(
            CredentialAttemptError::GenerationRejected
        ))
    );

    let second = credentials
        .reconfirm(CredentialReconfirmation::acknowledge_same_credentials())
        .unwrap_or_else(|_| unreachable!("explicit Robot reconfirmation was rejected"));
    assert_eq!(second.get(), 2);
    assert!(credentials.begin_attempt().is_ok());

    let mut username = Vec::from(b"replacement-user".as_slice());
    let mut password = Vec::from(b"replacement-secret".as_slice());
    let third = credentials
        .rotate_from_mut_bytes(&mut username, &mut password)
        .unwrap_or_else(|_| unreachable!("Robot credential rotation was rejected"));
    assert_eq!(third.get(), 3);
    assert!(username.iter().all(|byte| *byte == 0));
    assert!(password.iter().all(|byte| *byte == 0));
    assert_eq!(
        credentials.reject_attempt(&first),
        Err(RobotCredentialStateError::Attempt(
            CredentialAttemptError::StaleGeneration
        ))
    );
    let current = credentials
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("replacement Robot attempt was rejected"));
    assert_eq!(
        credentials.try_with_attempt(&current, |username, password| {
            username == "replacement-user" && password == "replacement-secret"
        }),
        Ok(true)
    );
}

#[test]
fn foreign_attempts_cannot_use_or_close_equal_generation_credentials() {
    let owner_a = credentials();
    let owner_b = credentials();
    let foreign = owner_a
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("owner A Robot attempt was rejected"));

    assert_eq!(
        owner_b.try_with_attempt(&foreign, |_, _| ()),
        Err(RobotCredentialStateError::Attempt(
            CredentialAttemptError::ForeignState
        ))
    );
    assert_eq!(
        owner_b.reject_attempt(&foreign),
        Err(RobotCredentialStateError::Attempt(
            CredentialAttemptError::ForeignState
        ))
    );
    assert_eq!(
        owner_b.status(),
        (
            CredentialAttemptGeneration::INITIAL,
            CredentialAttemptStatus::Open
        )
    );
}

#[test]
fn rotation_remains_available_while_an_owned_attempt_is_outstanding() {
    let mut credentials = credentials();
    let stale = credentials
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("initial Robot attempt was rejected"));
    let mut username = Vec::from(b"rotated-user".as_slice());
    let mut password = Vec::from(b"rotated-secret".as_slice());

    let replacement = credentials
        .rotate_from_mut_bytes(&mut username, &mut password)
        .unwrap_or_else(|_| unreachable!("in-flight Robot rotation was rejected"));

    assert_eq!(replacement.get(), 2);
    assert_eq!(
        credentials.reject_attempt(&stale),
        Err(RobotCredentialStateError::Attempt(
            CredentialAttemptError::StaleGeneration
        ))
    );
    assert!(username.iter().all(|byte| *byte == 0));
    assert!(password.iter().all(|byte| *byte == 0));
    assert_eq!(
        credentials.status(),
        (replacement, CredentialAttemptStatus::Open)
    );
}

#[test]
fn rejected_rotation_keeps_the_existing_generation_and_secrets() {
    let mut credentials = credentials();
    let before = credentials.status();
    let mut username = Vec::from(b"bad:user".as_slice());
    let mut password = Vec::from(b"replacement-secret".as_slice());
    assert_eq!(
        credentials.rotate_from_mut_bytes(&mut username, &mut password),
        Err(RobotCredentialRotationError::Credential(
            RobotCredentialError::InvalidUsername
        ))
    );
    assert_eq!(credentials.status(), before);
    let attempt = credentials
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("existing Robot generation was closed"));
    assert_eq!(
        credentials.try_with_attempt(&attempt, |username, password| {
            username == "robot-user" && password == "replace-only-secret"
        }),
        Ok(true)
    );
}

#[test]
fn guarded_rotation_clears_sources_and_advances_once() {
    let mut credentials = credentials();
    let mut username = Vec::from(b"guarded-user".as_slice());
    let mut password = Vec::from(b"guarded-secret".as_slice());
    let generation = credentials
        .rotate_from_secret_buffers(
            SecretBuffer::new(&mut username),
            SecretBuffer::new(&mut password),
        )
        .unwrap_or_else(|_| unreachable!("guarded Robot rotation was rejected"));
    assert_eq!(generation.get(), 2);
    assert!(username.iter().all(|byte| *byte == 0));
    assert!(password.iter().all(|byte| *byte == 0));
    let attempt = credentials
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("guarded replacement generation was closed"));
    assert_eq!(
        credentials.try_with_attempt(&attempt, |username, password| {
            username == "guarded-user" && password == "guarded-secret"
        }),
        Ok(true)
    );
}

#[test]
fn scope_is_fixed_to_robot_and_rejects_cloud_cross_use() {
    let scope = RobotCredentialScope;
    let authentication = scope
        .authentication_scope()
        .unwrap_or_else(|_| unreachable!("official Robot endpoint was rejected"));
    let policy = scope
        .authentication_policy()
        .unwrap_or_else(|_| unreachable!("official Robot policy was rejected"));
    assert_eq!(policy.validate(authentication), Ok(()));
    assert_eq!(
        policy.provider_requirement(),
        ScopeRequirement::Required(HETZNER_PROVIDER_ID)
    );
    assert_eq!(
        policy.service_requirement(),
        ScopeRequirement::Required(ROBOT_SERVICE_ID)
    );

    let cloud_policy = cloud_sdk::authentication::AuthenticationScopePolicy::new(
        ScopeRequirement::Required(HETZNER_PROVIDER_ID),
        ScopeRequirement::Required(CLOUD_SERVICE_ID),
        policy.endpoint_requirement(),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let error = cloud_policy
        .validate(authentication)
        .err()
        .unwrap_or_else(|| unreachable!("Robot credential entered Cloud scope"));
    assert_eq!(error.violation(), ScopeViolation::Mismatch);
}

#[test]
fn diagnostics_are_payload_free_and_status_is_public() {
    let credentials = credentials();
    let debug = format!("{credentials:?}");
    assert!(debug.contains("RobotCredentials"));
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("robot-user"));
    assert!(!debug.contains("replace-only-secret"));
    assert_eq!(
        credentials.status(),
        (
            CredentialAttemptGeneration::INITIAL,
            CredentialAttemptStatus::Open
        )
    );
}

#[test]
fn credential_owner_is_send_sync_but_not_clone() {
    fn assert_send_sync<T: Send + Sync>() {}
    fn assert_static<T: 'static>() {}
    assert_send_sync::<RobotCredentials>();
    assert_send_sync::<RobotCredentialAttempt>();
    assert_static::<RobotCredentialAttempt>();
}
