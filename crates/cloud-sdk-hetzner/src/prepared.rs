//! Complete fixed-buffer Hetzner operation preparation.

mod endpoints;
mod error;
mod operation;

pub use error::HetznerPreparationError;
pub use operation::{HetznerPreparedOperation, NoBody, NoQuery};

pub(crate) use operation::{EndpointWire, QueryWire, RequestShape, ResponseProfile};

#[cfg(test)]
mod tests;
