//! Compile-time operation associations for the complete active Hetzner API.
//!
//! The types in this module add nominal operation identity to the existing
//! source-locked request encoders. They do not add allocation, transport, or
//! runtime dependencies.

mod components;
mod markers;
mod policy;
mod prepared;
mod types;
mod validation;

pub use components::{
    AssociationError, BodyComponent, BodyFor, EndpointComponent, EndpointFor, QueryComponent,
    QueryFor,
};
pub use markers::{ALL_OPERATIONS, operations};
pub use policy::{
    AuthenticationClass, BodyPolicy, HetznerOperation, OperationAssociation, OperationDescriptor,
    PaginationPolicy, PermitClass, QueryPolicy, ResponseShape, RetryPolicy,
};
pub use prepared::{AssociatedOperation, AssociatedPreparationError, Prepared};
pub use types::*;

#[cfg(test)]
mod tests;
