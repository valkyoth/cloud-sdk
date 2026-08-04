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
mod dynamic;
mod metadata;
mod mock;
mod prepared;
mod raw_fault;
mod recording;
mod response;
mod script;
mod stream;

pub use adversarial::{
    AdversarialFixture, AdversarialKind, DEFAULT_RESPONSE_LIMIT, adversarial_corpus,
};
pub use body::{FixtureBody, FixtureBodyError, MAX_FIXTURE_BODY_BYTES};
pub use dynamic::{
    DynamicMockConfigError, DynamicMockError, DynamicMockTransport, DynamicRequest,
    DynamicResponder, ProviderFixtureBuilder,
};
pub use metadata::{
    ActionFixture, ActionState, FixtureMetadataError, PaginationFixture, RateLimitFixture,
};
pub use mock::{ExpectedRequest, LocalMockTransport, MockError, MockExchange, MockTransport};
pub use prepared::PreparedRequestRecord;
pub use raw_fault::{RawFault, RawFaultError, RawFaultExecutor};
pub use recording::{MAX_DYNAMIC_RECORDS, RecordedMethod, RecordedRequest, RequestRecordSlot};
pub use response::{FixtureKind, ResponseFixture, ResponseFixtureError};
pub use script::{ActionScript, PaginationScript, ScenarioScriptError};
pub use stream::{
    MAX_STREAM_FIXTURE_CHUNKS, StreamFixtureError, StreamFixtureSink, StreamFixtureSource,
    StreamPattern, StreamPatternSource,
};

#[cfg(test)]
mod tests;
