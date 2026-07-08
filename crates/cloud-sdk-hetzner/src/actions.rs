//! Shared action domains.

/// Action lifecycle state returned by long-running Hetzner operations.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ActionStatus {
    /// Action is running.
    Running,
    /// Action completed successfully.
    Success,
    /// Action failed.
    Error,
}
