#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(test)]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod adversarial;
mod body;
mod metadata;
mod mock;
mod response;

pub use adversarial::{
    AdversarialFixture, AdversarialKind, DEFAULT_RESPONSE_LIMIT, adversarial_corpus,
};
pub use body::{FixtureBody, FixtureBodyError, MAX_FIXTURE_BODY_BYTES};
pub use metadata::{
    ActionFixture, ActionState, FixtureMetadataError, PaginationFixture, RateLimitFixture,
};
pub use mock::{ExpectedRequest, MockError, MockExchange, MockTransport};
pub use response::{FixtureKind, ResponseFixture, ResponseFixtureError};

#[cfg(test)]
mod tests;
