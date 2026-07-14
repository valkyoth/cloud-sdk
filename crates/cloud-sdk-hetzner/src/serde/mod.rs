//! Optional no_std Serde boundary.
//!
//! Request serialization is exposed through checked wrappers so endpoint
//! selectors cannot leak into JSON bodies and aggregate size policy is applied
//! before a request becomes serializable. Response deserialization validates
//! security-relevant fields after parsing.

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
