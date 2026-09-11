//! Checked crates.io response admission and request scheduling foundations.
//!
//! No request is retried or sent by this module. Operation-specific clients
//! must bind these policies to their request metadata before execution.

#[cfg(feature = "alloc")]
mod envelope;
mod error;
mod rate;
#[cfg(feature = "alloc")]
mod response;
#[cfg(feature = "std")]
mod shared_rate;
mod user_agent;

pub use error::{CratesIoWireError, ProviderError, ProviderErrorKind};
pub use rate::{API_REQUEST_INTERVAL, ApiSchedule, MAX_PROVIDER_DELAY, ScheduleError};
#[cfg(feature = "alloc")]
pub use response::{JsonResponsePolicy, JsonSuccess, MAX_JSON_RESPONSE_BYTES};
#[cfg(any(feature = "blocking", feature = "async"))]
pub(crate) use shared_rate::OfficialApiAttempt;
#[cfg(feature = "std")]
pub use shared_rate::{OfficialApiGate, OfficialCallError};
#[cfg(all(test, feature = "std"))]
pub(crate) use shared_rate::{TEST_GATE_LOCK, reset_test_gate};
pub use user_agent::{IdentifyingUserAgent, MAX_USER_AGENT_BYTES, UserAgentError};

#[cfg(all(test, feature = "alloc"))]
mod boundary_tests;
#[cfg(test)]
mod policy_tests;
#[cfg(all(test, feature = "alloc"))]
mod response_tests;
