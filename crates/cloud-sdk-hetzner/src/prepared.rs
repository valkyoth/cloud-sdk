//! Complete fixed-buffer Hetzner operation preparation.

mod bodies;
mod endpoints;
mod error;
mod json;
mod operation;

pub use error::HetznerPreparationError;
pub use operation::{HetznerPreparedOperation, NoBody, NoQuery};

pub(crate) use json::JsonWriter;
pub(crate) use operation::{BodyWire, EndpointWire, QueryWire, RequestShape, ResponseProfile};

#[cfg(test)]
mod tests;
