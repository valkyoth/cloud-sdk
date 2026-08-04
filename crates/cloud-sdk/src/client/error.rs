use core::fmt;

use crate::operation::PreparedExecutionError;

/// Failure across preparation, authenticated execution, or checked decoding.
pub enum ClientExecutionError<P, T, D> {
    /// Provider request preparation failed before transport access.
    Preparation(P),
    /// Endpoint, authorization, transport, or response admission failed.
    Execution(PreparedExecutionError<T>),
    /// Provider-specific checked decoding failed.
    Decode(D),
}

impl<P, T, D> fmt::Debug for ClientExecutionError<P, T, D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Preparation(_) => "ClientExecutionError::Preparation([redacted])",
            Self::Execution(_) => "ClientExecutionError::Execution([redacted])",
            Self::Decode(_) => "ClientExecutionError::Decode([redacted])",
        })
    }
}

impl<P, T, D> fmt::Display for ClientExecutionError<P, T, D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Preparation(_) => "client request preparation failed",
            Self::Execution(_) => "client request execution failed",
            Self::Decode(_) => "client response decoding failed",
        })
    }
}

impl<P, T, D> core::error::Error for ClientExecutionError<P, T, D> {}
