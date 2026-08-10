//! Compile-time operation associations for the complete active Hetzner API.
//!
//! The types in this module add nominal operation identity to the existing
//! source-locked request encoders. They do not add allocation, transport, or
//! runtime dependencies.

#[cfg(feature = "serde")]
mod client;
mod components;
mod evidence;
mod identity;
mod markers;
mod permit;
mod policy;
mod prepared;
mod types;
pub(crate) mod validation;

#[cfg(feature = "serde")]
pub use client::HetznerClientOperation;
pub use components::{
    AssociationError, BodyComponent, BodyFor, EndpointComponent, EndpointFor, QueryComponent,
    QueryFor,
};
#[doc(hidden)]
pub use evidence::{EvidenceLabel, OperationBindingEvidence};
pub use markers::{ALL_OPERATION_EVIDENCE, ALL_OPERATIONS, operations};
#[cfg(feature = "serde")]
pub use permit::Sha256PlanHasher;
pub use permit::{
    AssociatedCanonicalPlanFingerprint, AssociatedCostPermit, AssociatedDestructivePermit,
    AssociatedMutationPermit, AssociatedPermitAttempt, AssociatedPlanConfirmation,
    AssociatedPlanFingerprintDigest, AssociatedPlanSubject, AssociatedSharedCostPermit,
    AssociatedSharedDestructivePermit, AssociatedSharedMutationPermit,
    build_associated_canonical_plan, build_associated_plan_digest,
};
pub use policy::{
    AuthenticationClass, BodyPolicy, HetznerOperation, OperationAssociation, OperationDescriptor,
    PaginationPolicy, PermitClass, QueryPolicy, ReadOnlyOperation, ResponseIdentityClass,
    ResponseShape, RetryPolicy,
};
pub use prepared::{
    AssociatedCheckedResponse, AssociatedOperation, AssociatedPreparationError, Prepared,
};
pub use types::*;

pub(crate) use identity::ExpectedResponseIdentity;
pub(crate) use markers::operation_path_template;

#[cfg(test)]
mod identity_contract_tests;
#[cfg(test)]
mod tests;
