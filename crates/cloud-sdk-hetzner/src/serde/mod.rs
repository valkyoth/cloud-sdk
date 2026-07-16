//! Optional no_std Serde boundary.
//!
//! Current public request serialization is limited to checked RRSet body
//! wrappers, so endpoint selectors cannot leak into those JSON bodies and an
//! aggregate size policy is applied before serialization. Checked response
//! decoding binds every active operation to its source-locked status and
//! envelope shape, validates security-relevant fields, and returns typed
//! actions, resource identities, pagination, special results, or API errors.
//! Provider-complete resource field models remain future work.

mod binding;
mod checked;
mod models;
mod pagination;
mod response;
mod rrsets;
mod strict_json;

pub use checked::{CheckedHetznerResponse, HetznerApiError, HetznerDecodeError, decode_response};
pub use models::{
    ActionResult, ActionResultError, ActionResultResource, CompositeResult, FolderList,
    HetznerSuccess, MetricPoint, MetricSeries, Metrics, NamedSensitiveText, Pricing, Resource,
    ResourceIdentifier, ResourceKind, ResponseModelError, SensitiveText, ZoneFile,
};
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
mod checked_tests;
#[cfg(test)]
mod tests;
