use core::fmt;

use super::PermitDisposition;

/// Prepared execution failure plus the fail-closed permit disposition.
pub struct PermitExecutionError<E> {
    pub(crate) execution: super::super::PreparedExecutionError<E>,
    pub(crate) disposition: PermitDisposition,
}

impl<E> PermitExecutionError<E> {
    /// Returns the underlying payload-redacting execution failure.
    #[must_use]
    pub const fn execution(&self) -> &super::super::PreparedExecutionError<E> {
        &self.execution
    }

    /// Returns the recovery or reconciliation disposition.
    #[must_use]
    pub const fn disposition(&self) -> PermitDisposition {
        self.disposition
    }
}

impl<E> fmt::Debug for PermitExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PermitExecutionError")
            .field("execution", &"[redacted]")
            .field("disposition", &self.disposition)
            .finish()
    }
}

impl<E> fmt::Display for PermitExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("permit-authorized request execution failed")
    }
}

impl<E> core::error::Error for PermitExecutionError<E> {}
