use cloud_sdk::operation::{
    AttemptBudget, ExecutionPermitError, PermitContext, PermitTimestamp, PermitValidity,
    PlanChange, PlanFingerprintBuildError, PlanFingerprintScope, PreparationStorage, ReplayPolicy,
};
use cloud_sdk::transport::EndpointIdentity;

use super::*;
use crate::association::Sha256PlanHasher;
use crate::endpoint::official_robot_endpoint_identity;
use crate::robot::RobotCancellationSchedule;

use super::tests::{id, name, selector, vlan};

#[test]
fn sensitive_mutation_requires_digest_and_correct_authority() {
    let request = RobotVSwitchCreateRequest::new(name("fabric"), vlan(4000));
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("create preparation failed"));
    let mut exact = [0xa5_u8; 4_096];
    assert!(matches!(
        build_robot_vswitch_canonical_plan(plan(prepared), &mut exact),
        Err(PlanFingerprintBuildError::SensitiveBodyRequiresDigest)
    ));
    assert_eq!(exact, [0_u8; 4_096]);

    let mut target = [0_u8; 128];
    let mut body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("create preparation failed"));
    let mut scratch = [0xa5_u8; 4_096];
    let mut digest = [0x5a_u8; 32];
    let fingerprint = build_robot_vswitch_plan_digest(
        plan(prepared),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("vSwitch digest failed"));
    assert_eq!(scratch, [0_u8; 4_096]);
    assert!(
        RobotVSwitchMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100),)
            .is_ok()
    );
    assert!(matches!(
        RobotVSwitchDestructivePermit::new(
            fingerprint.subject(),
            PermitTimestamp::from_seconds(100),
        ),
        Err(ExecutionPermitError::ScopeMismatch)
    ));
}

#[test]
fn destructive_operations_reject_mutation_authority() {
    let requests = [selector("321")];
    let servers = RobotVSwitchServers::new(&requests)
        .unwrap_or_else(|_| unreachable!("membership fixture failed"));
    let remove = RobotVSwitchRemoveServersRequest::new(id(), servers);
    assert_destructive(&remove);

    let cancel = RobotVSwitchCancelRequest::new(id(), RobotCancellationSchedule::Immediate);
    assert_destructive(&cancel);
}

fn assert_destructive<R>(request: &R)
where
    R: PrepareBound,
{
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("destructive preparation failed"));
    let mut scratch = [0_u8; 4_096];
    let mut digest = [0_u8; 32];
    let fingerprint = build_robot_vswitch_plan_digest(
        plan(prepared),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("destructive digest failed"));
    assert!(
        RobotVSwitchDestructivePermit::new(
            fingerprint.subject(),
            PermitTimestamp::from_seconds(100),
        )
        .is_ok()
    );
    assert!(matches!(
        RobotVSwitchMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100),),
        Err(ExecutionPermitError::ScopeMismatch)
    ));
}

trait PrepareBound: RobotVSwitchPermitRequest {
    fn prepare_bound<'storage, 'request>(
        &'request self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRobotVSwitch<'storage, 'request, Self>, RobotVSwitchRequestError>
    where
        Self: Sized;
}

macro_rules! prepare_bound_impl {
    ($($type:ty),+ $(,)?) => {$ (
        impl PrepareBound for $type {
            fn prepare_bound<'storage, 'request>(
                &'request self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRobotVSwitch<'storage, 'request, Self>, RobotVSwitchRequestError> {
                <$type>::prepare_bound(self, storage)
            }
        }
    )+ };
}

prepare_bound_impl!(
    RobotVSwitchCancelRequest,
    RobotVSwitchRemoveServersRequest<'_>,
);

fn plan<'storage, 'request, R>(
    prepared: PreparedRobotVSwitch<'storage, 'request, R>,
) -> RobotVSwitchPlanConfirmation<'static, 'storage, 'request, R>
where
    R: RobotVSwitchPermitRequest,
{
    RobotVSwitchPlanConfirmation::new(
        prepared,
        endpoint(),
        PlanFingerprintScope::Value(b"robot-account"),
        PlanFingerprintScope::Absent,
        PermitContext::new(b"v0.90 Robot vSwitch fixture")
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
