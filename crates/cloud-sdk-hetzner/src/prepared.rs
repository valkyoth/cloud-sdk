//! Complete fixed-buffer Hetzner operation preparation.

mod bodies;
mod endpoints;
mod error;
mod json;
mod operation;
mod wire_policy;

pub use error::HetznerPreparationError;
pub use operation::{HetznerPreparedOperation, NoBody, NoQuery};

pub(crate) use json::{JsonWriter, SensitiveJsonString, encode_object};
pub(crate) use operation::{
    BodyWire, EndpointWire, QueryWire, RequestShape, ResponseProfile, prepare_parts,
};
pub(crate) use wire_policy::authentication_policy;

#[cfg(test)]
mod response_metadata_tests;
#[cfg(test)]
mod tests;
