//! Read-only Network membership adapter.

use cloud_sdk::operation::CostIntent;

use super::super::{RequestShape, ResponseProfile};
use crate::cloud::networks::NetworkMembersEndpoint;
use crate::prepared::operation::OperationClass;

endpoint_wire!(
    NetworkMembersEndpoint,
    endpoint => RequestShape::OptionalQuery,
    ResponseProfile::JsonOk,
    match endpoint {
        _ => "list_network_members",
    },
    OperationClass::ReadOnly,
    CostIntent::NoKnownCost,
    identity endpoint => { let _ = endpoint; crate::association::ExpectedResponseIdentity::None }
);
