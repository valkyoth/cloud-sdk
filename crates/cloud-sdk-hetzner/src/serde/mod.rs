//! Optional no_std Serde boundary.
//!
//! Current public request serialization is limited to checked RRSet body
//! wrappers, so endpoint selectors cannot leak into those JSON bodies and an
//! aggregate size policy is applied before serialization. Current response
//! deserialization covers shared pagination, action, and API error envelopes
//! and validates security-relevant fields after parsing. Resource-specific
//! bodies and responses are not implied by enabling this feature.

mod pagination;
mod response;
mod rrsets;

pub use pagination::PaginationEnvelope;
pub use response::{
    ActionEnvelope, ActionResource, ActionResponse, ApiErrorEnvelope, ApiErrorResponse,
    MAX_ACTION_RESPONSE_RESOURCES, MAX_API_ERROR_MESSAGE_BYTES, MAX_SERDE_RESPONSE_BYTES,
    ResponseBytes, ResponseSizeError,
};
pub use rrsets::{MAX_RRSET_JSON_BODY_BYTES, RrsetBodyError, RrsetRequestBody};

#[cfg(test)]
mod adversarial_tests;
#[cfg(test)]
mod tests;
