#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(test)]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

macro_rules! impl_static_error {
    ($error:ty, $($pattern:pat => $message:literal),+ $(,)?) => {
        impl core::fmt::Display for $error {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(match self {
                    $($pattern => $message,)+
                })
            }
        }

        impl core::error::Error for $error {}
    };
}

mod adversarial;
mod body;
mod metadata;
mod mock;
mod prepared;
mod response;

pub use adversarial::{
    AdversarialFixture, AdversarialKind, DEFAULT_RESPONSE_LIMIT, adversarial_corpus,
};
pub use body::{FixtureBody, FixtureBodyError, MAX_FIXTURE_BODY_BYTES};
pub use metadata::{
    ActionFixture, ActionState, FixtureMetadataError, PaginationFixture, RateLimitFixture,
};
pub use mock::{ExpectedRequest, MockError, MockExchange, MockTransport};
pub use prepared::PreparedRequestRecord;
pub use response::{FixtureKind, ResponseFixture, ResponseFixtureError};

#[cfg(test)]
mod tests;
