//! Official-endpoint Robot client and sealed operation contract.

mod construction;
mod execution;
mod inventory;
mod mutation;
mod operation;
mod operations;
mod permit;

#[cfg(test)]
mod coverage_tests;

pub use construction::{RobotClient, RobotClientConstructionError, RobotClientLifecycleError};
pub use execution::RobotClientExecutionError;
pub use inventory::{ROBOT_CLIENT_METHODS, RobotClientMethodDescriptor};
pub use mutation::{
    PreparedRobotClientMutation, RobotClientMutationOperation,
    RobotMutationCanonicalPlanFingerprint, RobotMutationClientExecutionError,
    RobotMutationDestructivePermit, RobotMutationPermit, RobotMutationPermitAttempt,
    RobotMutationPlanConfirmation, RobotMutationPlanFingerprintDigest, RobotMutationPlanSubject,
    RobotMutationSharedDestructivePermit, RobotMutationSharedPermit,
    build_robot_mutation_canonical_plan, build_robot_mutation_plan_digest,
    prepare_robot_client_mutation,
};
pub use operation::{
    RobotClientOperation, RobotClientResponse, RobotDirectClientOperation, RobotResponseDecodeError,
};
pub use permit::{RobotClientAttempt, RobotPermitClientExecutionError};
