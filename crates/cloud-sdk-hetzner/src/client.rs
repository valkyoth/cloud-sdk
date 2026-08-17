//! Service-typed Hetzner client construction over provider-neutral transports.

#[cfg(feature = "serde")]
mod cloud;
mod construction;
#[cfg(feature = "serde")]
mod dns;
#[cfg(feature = "serde")]
mod execution;
#[cfg(feature = "serde")]
mod robot;
#[cfg(feature = "serde")]
mod security;
#[cfg(feature = "serde")]
mod storage;

#[cfg(feature = "serde")]
pub use cloud::{CLOUD_CLIENT_METHODS, CloudClientMethodDescriptor, CloudReadResult};
pub use construction::{
    CloudClient, CustomEndpointTrust, DnsClient, EndpointTrust, HetznerClient,
    HetznerClientConstructionError, OfficialEndpointTrust, SecurityClient, StorageClient,
};
#[cfg(feature = "serde")]
pub use dns::{DNS_CLIENT_METHODS, DnsClientMethodDescriptor, DnsReadResult};
#[cfg(feature = "serde")]
pub use robot::{
    PreparedRobotClientMutation, ROBOT_CLIENT_METHODS, RobotClient, RobotClientAttempt,
    RobotClientConstructionError, RobotClientExecutionError, RobotClientLifecycleError,
    RobotClientMethodDescriptor, RobotClientMutationOperation, RobotClientOperation,
    RobotClientResponse, RobotDirectClientOperation, RobotMutationCanonicalPlanFingerprint,
    RobotMutationClientExecutionError, RobotMutationDestructivePermit, RobotMutationPermit,
    RobotMutationPermitAttempt, RobotMutationPlanConfirmation, RobotMutationPlanFingerprintDigest,
    RobotMutationPlanSubject, RobotMutationSharedDestructivePermit, RobotMutationSharedPermit,
    RobotPermitClientExecutionError, RobotResponseDecodeError, build_robot_mutation_canonical_plan,
    build_robot_mutation_plan_digest, prepare_robot_client_mutation,
};
#[cfg(feature = "serde")]
pub use security::{SECURITY_CLIENT_METHODS, SecurityClientMethodDescriptor, SecurityReadResult};
#[cfg(feature = "serde")]
pub use storage::{STORAGE_CLIENT_METHODS, StorageClientMethodDescriptor, StorageReadResult};

#[cfg(test)]
mod tests;
