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

impl ActionStatus {
    /// Parses an action status string.
    #[must_use]
    pub const fn from_api_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"running" => Some(Self::Running),
            b"success" => Some(Self::Success),
            b"error" => Some(Self::Error),
            _ => None,
        }
    }

    /// Returns the API status string.
    #[must_use]
    pub const fn as_api_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Success => "success",
            Self::Error => "error",
        }
    }

    /// Returns true when polling should stop.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        match self {
            Self::Running => false,
            Self::Success | Self::Error => true,
        }
    }
}

/// Action identifier returned by Hetzner action resources.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActionId(u64);

impl ActionId {
    /// Creates a nonzero action identifier.
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            return None;
        }
        Some(Self(value))
    }

    /// Returns the raw identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{ActionId, ActionStatus};

    #[test]
    fn parses_action_status() {
        assert_eq!(
            ActionStatus::from_api_str("running"),
            Some(ActionStatus::Running)
        );
        assert_eq!(ActionStatus::from_api_str("unknown"), None);
        assert!(!ActionStatus::Running.is_terminal());
        assert!(ActionStatus::Success.is_terminal());
        assert!(ActionStatus::Error.is_terminal());
    }

    #[test]
    fn rejects_zero_action_ids() {
        assert_eq!(ActionId::new(0), None);
        assert_eq!(ActionId::new(42).map(ActionId::get), Some(42));
    }
}
