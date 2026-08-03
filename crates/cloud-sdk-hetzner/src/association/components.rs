//! Nominal endpoint, query, and body wrappers.

use core::fmt;
use core::marker::PhantomData;

use super::policy::{BodyPolicy, HetznerOperation, QueryPolicy};
use crate::prepared::{BodyWire, EndpointWire, NoBody, NoQuery, QueryWire};

mod endpoint_private {
    pub trait Sealed {}
}
mod query_private {
    pub trait Sealed {}
}
mod body_private {
    pub trait Sealed {}
}

/// Provider-owned endpoint component admitted by typed association wrappers.
#[doc(hidden)]
pub trait EndpointComponent: endpoint_private::Sealed + Copy {
    /// Returns the exact source-locked operation key.
    #[doc(hidden)]
    fn operation_key(self) -> &'static str;
}

impl<T: EndpointWire> endpoint_private::Sealed for T {}
impl<T: EndpointWire> EndpointComponent for T {
    fn operation_key(self) -> &'static str {
        EndpointWire::operation_key(self)
    }
}

/// Provider-owned query component admitted by typed association wrappers.
#[doc(hidden)]
pub trait QueryComponent: query_private::Sealed + Copy {
    /// Reports whether this query belongs to an operation key.
    #[doc(hidden)]
    fn accepts_operation(self, operation_key: &str) -> bool;
}

impl<T: QueryWire> query_private::Sealed for T {}
impl<T: QueryWire> QueryComponent for T {
    fn accepts_operation(self, operation_key: &str) -> bool {
        QueryWire::accepts_operation(self, operation_key)
    }
}

/// Provider-owned body component admitted by typed association wrappers.
#[doc(hidden)]
pub trait BodyComponent: body_private::Sealed + Copy {
    /// Reports whether this body belongs to an operation key.
    #[doc(hidden)]
    fn accepts_operation(self, operation_key: &str) -> bool;
}

impl<T: BodyWire> body_private::Sealed for T {}
impl<T: BodyWire> BodyComponent for T {
    fn accepts_operation(self, operation_key: &str) -> bool {
        BodyWire::accepts_operation(self, operation_key)
    }
}

/// Failure while binding a concrete request component to an operation marker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssociationError {
    /// The endpoint identifies another operation.
    EndpointMismatch,
    /// The operation forbids a query.
    QueryForbidden,
    /// The operation requires a query.
    QueryRequired,
    /// The query is not admitted by this operation.
    QueryMismatch,
    /// The operation forbids a request body.
    BodyForbidden,
    /// The operation requires a JSON request body.
    BodyRequired,
    /// The request body is not admitted by this operation.
    BodyMismatch,
    /// Prepared runtime policy differs from the compile-time association.
    PreparedPolicyMismatch,
}

impl fmt::Display for AssociationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EndpointMismatch => "endpoint does not match operation association",
            Self::QueryForbidden => "operation association forbids a query",
            Self::QueryRequired => "operation association requires a query",
            Self::QueryMismatch => "query does not match operation association",
            Self::BodyForbidden => "operation association forbids a request body",
            Self::BodyRequired => "operation association requires a request body",
            Self::BodyMismatch => "request body does not match operation association",
            Self::PreparedPolicyMismatch => {
                "prepared runtime policy does not match operation association"
            }
        })
    }
}

impl core::error::Error for AssociationError {}

/// Endpoint value bound to exactly one operation marker.
pub struct EndpointFor<O, E> {
    inner: E,
    operation: PhantomData<fn() -> O>,
}

impl<O, E: Copy> Copy for EndpointFor<O, E> {}
impl<O, E: Copy> Clone for EndpointFor<O, E> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<O, E: PartialEq> PartialEq for EndpointFor<O, E> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<O, E: Eq> Eq for EndpointFor<O, E> {}

impl<O: HetznerOperation, E: EndpointComponent> EndpointFor<O, E> {
    /// Validates and binds an endpoint to `O`.
    pub fn try_new(endpoint: E) -> Result<Self, AssociationError> {
        if endpoint.operation_key() != O::DESCRIPTOR.operation_id().as_str() {
            return Err(AssociationError::EndpointMismatch);
        }
        Ok(Self {
            inner: endpoint,
            operation: PhantomData,
        })
    }

    /// Returns the bound endpoint value.
    #[must_use]
    pub const fn into_inner(self) -> E {
        self.inner
    }
}

impl<O: HetznerOperation, E> fmt::Debug for EndpointFor<O, E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EndpointFor")
            .field("operation", &O::DESCRIPTOR.operation_id())
            .field("endpoint", &"[bound]")
            .finish()
    }
}

/// Query value bound to exactly one operation marker.
pub struct QueryFor<O, Q> {
    inner: Q,
    operation: PhantomData<fn() -> O>,
}

impl<O, Q: Copy> Copy for QueryFor<O, Q> {}
impl<O, Q: Copy> Clone for QueryFor<O, Q> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<O, Q: PartialEq> PartialEq for QueryFor<O, Q> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<O, Q: Eq> Eq for QueryFor<O, Q> {}

impl<O: HetznerOperation, Q: QueryComponent> QueryFor<O, Q> {
    /// Validates and binds a concrete query to `O`.
    pub fn try_new(query: Q) -> Result<Self, AssociationError> {
        if matches!(O::DESCRIPTOR.query_policy(), QueryPolicy::Forbidden) {
            return Err(AssociationError::QueryForbidden);
        }
        if !query.accepts_operation(O::DESCRIPTOR.operation_id().as_str()) {
            return Err(AssociationError::QueryMismatch);
        }
        Ok(Self {
            inner: query,
            operation: PhantomData,
        })
    }

    /// Returns the bound query value.
    #[must_use]
    pub const fn into_inner(self) -> Q {
        self.inner
    }
}

impl<O: HetznerOperation> QueryFor<O, NoQuery> {
    pub(crate) fn none() -> Result<Self, AssociationError> {
        if matches!(O::DESCRIPTOR.query_policy(), QueryPolicy::Required) {
            return Err(AssociationError::QueryRequired);
        }
        Ok(Self {
            inner: NoQuery,
            operation: PhantomData,
        })
    }
}

impl<O: HetznerOperation, Q> fmt::Debug for QueryFor<O, Q> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("QueryFor")
            .field("operation", &O::DESCRIPTOR.operation_id())
            .field("query", &"[redacted]")
            .finish()
    }
}

/// Request body bound to exactly one operation marker.
pub struct BodyFor<O, B> {
    inner: B,
    operation: PhantomData<fn() -> O>,
}

impl<O, B: Copy> Copy for BodyFor<O, B> {}
impl<O, B: Copy> Clone for BodyFor<O, B> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<O, B: PartialEq> PartialEq for BodyFor<O, B> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<O, B: Eq> Eq for BodyFor<O, B> {}

impl<O: HetznerOperation, B: BodyComponent> BodyFor<O, B> {
    /// Validates and binds a concrete request body to `O`.
    pub fn try_new(body: B) -> Result<Self, AssociationError> {
        if matches!(O::DESCRIPTOR.body_policy(), BodyPolicy::Forbidden) {
            return Err(AssociationError::BodyForbidden);
        }
        if !body.accepts_operation(O::DESCRIPTOR.operation_id().as_str()) {
            return Err(AssociationError::BodyMismatch);
        }
        Ok(Self {
            inner: body,
            operation: PhantomData,
        })
    }

    /// Returns the bound body value.
    #[must_use]
    pub const fn into_inner(self) -> B {
        self.inner
    }
}

impl<O: HetznerOperation> BodyFor<O, NoBody> {
    pub(crate) fn none() -> Result<Self, AssociationError> {
        if matches!(O::DESCRIPTOR.body_policy(), BodyPolicy::RequiredJson) {
            return Err(AssociationError::BodyRequired);
        }
        Ok(Self {
            inner: NoBody,
            operation: PhantomData,
        })
    }
}

impl<O: HetznerOperation, B> fmt::Debug for BodyFor<O, B> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BodyFor")
            .field("operation", &O::DESCRIPTOR.operation_id())
            .field("body", &"[redacted]")
            .finish()
    }
}
