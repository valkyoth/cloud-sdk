use super::{PlanConfirmation, PlanFingerprintBuildError, PlanFingerprintRef, PlanSubject};
use crate::operation::{CostIntent, OperationImpact, PermitScope, PlanChange, ReplayPolicy};

pub(super) fn validate<E>(
    plan: &PlanConfirmation<'_, '_>,
    authorization_evidence_supplied: bool,
) -> Result<PermitScope, PlanFingerprintBuildError<E>> {
    if plan.prepared.authorization_evidence_required() && !authorization_evidence_supplied {
        return Err(PlanFingerprintBuildError::AuthorizationEvidenceRequired);
    }
    if plan.prepared.operation_id().is_none() {
        return Err(PlanFingerprintBuildError::MissingOperationId);
    }
    if !plan
        .prepared
        .service()
        .endpoint_policy()
        .admits(plan.endpoint)
    {
        return Err(PlanFingerprintBuildError::EndpointNotAdmitted);
    }
    if plan.change == PlanChange::NoOp {
        return Err(PlanFingerprintBuildError::NoOp);
    }
    plan.account
        .bytes()
        .map_err(PlanFingerprintBuildError::Context)?;
    plan.tenant
        .bytes()
        .map_err(PlanFingerprintBuildError::Context)?;
    let metadata = plan.prepared.metadata();
    let scope = match (metadata.cost_intent(), metadata.impact()) {
        (CostIntent::MayIncurCost, _) => PermitScope::Cost,
        (_, OperationImpact::Destructive) => PermitScope::Destructive,
        (_, OperationImpact::Mutation) => PermitScope::Mutation,
        (_, OperationImpact::ReadOnly) => return Err(PlanFingerprintBuildError::ReadOnlyOperation),
    };
    match (scope, plan.cost) {
        (PermitScope::Cost, None) => return Err(PlanFingerprintBuildError::MissingCost),
        (PermitScope::Mutation | PermitScope::Destructive, Some(_)) => {
            return Err(PlanFingerprintBuildError::UnexpectedCost);
        }
        _ => {}
    }
    match (plan.replay, plan.attempts.get(), plan.idempotency) {
        (ReplayPolicy::SingleAttempt, 1, None)
        | (ReplayPolicy::RecoverNotSent, _, None)
        | (ReplayPolicy::ReconcileThenRetry, _, Some(_)) => {}
        (ReplayPolicy::SingleAttempt, _, _) => {
            return Err(PlanFingerprintBuildError::InvalidSingleAttemptBudget);
        }
        (ReplayPolicy::ReconcileThenRetry, _, None) => {
            return Err(PlanFingerprintBuildError::MissingIdempotency);
        }
        (_, _, Some(_)) => return Err(PlanFingerprintBuildError::UnexpectedIdempotency),
    }
    Ok(scope)
}

pub(super) fn subject<'request, 'plan: 'fingerprint, 'fingerprint>(
    plan: &'fingerprint PlanConfirmation<'plan, 'request>,
    scope: PermitScope,
    fingerprint: PlanFingerprintRef<'fingerprint>,
) -> PlanSubject<'request, 'fingerprint> {
    PlanSubject {
        prepared: &plan.prepared,
        fingerprint,
        endpoint: plan.endpoint,
        scope,
        validity: plan.validity,
        replay: plan.replay,
        attempts: plan.attempts,
        idempotency: plan.idempotency,
    }
}
