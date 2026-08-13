use cloud_sdk::operation::{
    AttemptBudget, ExecutionPermitError, PermitContext, PermitTimestamp, PermitValidity,
    PlanChange, PlanFingerprintBuildError, PlanFingerprintScope, PreparationStorage, ReplayPolicy,
};
use cloud_sdk::transport::EndpointIdentity;

use super::*;
use crate::association::Sha256PlanHasher;
use crate::endpoint::official_robot_endpoint_identity;
use crate::robot::ssh_keys::tests::{data, fingerprint, name};

#[test]
fn sensitive_create_requires_a_strong_digest() {
    let request = RobotSshKeyCreateRequest::new(name("deploy-key"), data());
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 16_384];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("create preparation failed"));
    let mut exact = [0xa5_u8; 16_384];
    assert!(matches!(
        build_robot_ssh_key_canonical_plan(plan(prepared), &mut exact),
        Err(PlanFingerprintBuildError::SensitiveBodyRequiresDigest)
    ));
    assert_eq!(exact, [0_u8; 16_384]);

    let mut target = [0_u8; 128];
    let mut body = [0_u8; 16_384];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("create preparation failed"));
    let mut scratch = [0xa5_u8; 16_384];
    let mut digest = [0x5a_u8; 32];
    let update_fingerprint = build_robot_ssh_key_plan_digest(
        plan(prepared),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("create digest failed"));
    assert_eq!(scratch, [0_u8; 16_384]);
    assert!(
        RobotSshKeyMutationPermit::new(
            update_fingerprint.subject(),
            PermitTimestamp::from_seconds(100)
        )
        .is_ok()
    );
}

#[test]
fn mutation_and_destructive_authority_are_not_interchangeable() {
    let update = RobotSshKeyUpdateRequest::new(fingerprint(), name("renamed"));
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 256];
    let prepared = update
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("update preparation failed"));
    let mut scratch = [0_u8; 4_096];
    let mut digest = [0_u8; 32];
    let update_fingerprint = build_robot_ssh_key_plan_digest(
        plan(prepared),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("update digest failed"));
    assert!(matches!(
        RobotSshKeyDestructivePermit::new(
            update_fingerprint.subject(),
            PermitTimestamp::from_seconds(100)
        ),
        Err(ExecutionPermitError::ScopeMismatch)
    ));

    let delete = RobotSshKeyDeleteRequest::new(fingerprint());
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 1];
    let prepared = delete
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("delete preparation failed"));
    let mut exact = [0_u8; 4_096];
    let fingerprint = build_robot_ssh_key_canonical_plan(plan(prepared), &mut exact)
        .unwrap_or_else(|_| unreachable!("delete plan failed"));
    assert!(matches!(
        RobotSshKeyMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100)),
        Err(ExecutionPermitError::ScopeMismatch)
    ));
}

fn plan<'storage, 'request, R>(
    prepared: PreparedRobotSshKey<'storage, 'request, R>,
) -> RobotSshKeyPlanConfirmation<'static, 'storage, 'request, R>
where
    R: RobotSshKeyPermitRequest,
{
    RobotSshKeyPlanConfirmation::new(
        prepared,
        endpoint(),
        PlanFingerprintScope::Value(b"robot-account"),
        PlanFingerprintScope::Absent,
        PermitContext::new(b"v0.88 Robot SSH-key fixture")
            .unwrap_or_else(|_| unreachable!("permit context failed")),
        PermitValidity::new(
            PermitTimestamp::from_seconds(100),
            PermitTimestamp::from_seconds(200),
        )
        .unwrap_or_else(|_| unreachable!("permit validity failed")),
        ReplayPolicy::SingleAttempt,
        AttemptBudget::new(1).unwrap_or_else(|_| unreachable!("attempt budget failed")),
        PlanChange::ChangesState,
        None,
        None,
    )
}

fn endpoint() -> EndpointIdentity<'static> {
    official_robot_endpoint_identity()
        .unwrap_or_else(|_| unreachable!("official Robot endpoint failed"))
}
