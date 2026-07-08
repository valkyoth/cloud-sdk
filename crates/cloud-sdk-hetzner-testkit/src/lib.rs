#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "std")]
extern crate std;

/// Fixture category planned for the testkit.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FixtureKind {
    /// Pagination response fixture.
    Pagination,
    /// Action polling response fixture.
    Action,
    /// Error response fixture.
    Error,
}
