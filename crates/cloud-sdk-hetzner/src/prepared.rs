//! Complete fixed-buffer Hetzner operation preparation.

mod bodies;
mod endpoints;
mod error;
mod json;
mod operation;
mod path_template;
mod wire_policy;

pub use error::HetznerPreparationError;
pub use operation::{HetznerPreparedOperation, NoBody, NoQuery};

pub(crate) use json::{JsonWriter, SensitiveJsonString, encode_object};
pub(crate) use operation::{
    BodyWire, EndpointWire, QueryWire, RequestShape, ResponseProfile, clear_preparation_storage,
    prepare_parts_with_policy, response_policy,
};
pub(crate) use wire_policy::{authentication_policy, provider_service, raw_response_policy};

#[cfg(test)]
mod body_sensitivity_tests;
#[cfg(test)]
mod response_metadata_tests;
#[cfg(test)]
mod tests;
