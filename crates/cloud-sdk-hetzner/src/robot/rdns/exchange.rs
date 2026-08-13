use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PrepareOperation, PreparedRequest,
    ResponsePolicyError,
};
use cloud_sdk::transport::ResponseBuffer;

use super::decode::{RobotRdnsDecodeError, decode_robot_rdns, decode_robot_rdns_list};
use alloc::vec::Vec;
use core::net::IpAddr;

use super::model::{RobotRdns, RobotRdnsFilteredMembership, RobotRdnsList};
use super::request::*;
use crate::robot::RobotIpList;

/// Prepared Robot reverse-DNS request retaining its exact typed association.
pub struct PreparedRobotRdns<'storage, 'request, R> {
    request: &'request R,
    inner: PreparedRequest<'storage>,
}

impl<'storage, 'request, R> PreparedRobotRdns<'storage, 'request, R> {
    /// Borrows the provider-neutral prepared request for inspection.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'storage> {
        self.inner
    }

    /// Applies the exact response policy and retains request provenance.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedRobotRdns<'buffer, 'request, R>, ResponsePolicyError> {
        self.inner
            .validate_response(response)
            .map(|inner| CheckedRobotRdns {
                request: self.request,
                inner,
            })
    }

    pub(super) fn into_plan_parts(self) -> (PreparedRequest<'storage>, &'request R) {
        (self.inner, self.request)
    }
}

impl<R> core::fmt::Debug for PreparedRobotRdns<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PreparedRobotRdns")
            .field("request", &"[bound]")
            .field("prepared", &self.inner)
            .finish()
    }
}

/// Checked Robot reverse-DNS response retaining its admitting request.
pub struct CheckedRobotRdns<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
}

impl<'buffer, 'request, R> CheckedRobotRdns<'buffer, 'request, R> {
    pub(super) const fn from_executed(
        request: &'request R,
        inner: CheckedResponseGuard<'buffer>,
    ) -> Self {
        Self { request, inner }
    }
}

impl<R> core::fmt::Debug for CheckedRobotRdns<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CheckedRobotRdns")
            .field("request", &"[bound]")
            .field("response", &"[checked]")
            .finish()
    }
}

macro_rules! prepare_bound {
    ($($type:ty),+ $(,)?) => {$ (
        impl $type {
            /// Prepares this operation while retaining exact response association.
            pub fn prepare_bound<'storage, 'request>(
                &'request self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRobotRdns<'storage, 'request, Self>, RobotRdnsRequestError> {
                let inner = self.prepare(storage)?;
                Ok(PreparedRobotRdns { request: self, inner })
            }
        }
    )+ };
}

prepare_bound!(
    RobotRdnsListRequest,
    RobotRdnsGetRequest,
    RobotRdnsSetRequest,
    RobotRdnsUpdateRequest,
    RobotRdnsDeleteRequest,
);

impl CheckedRobotRdns<'_, '_, RobotRdnsListRequest> {
    /// Decodes an unfiltered bounded list of entries with distinct addresses.
    ///
    /// A filtered Robot response does not echo its server association. Use
    /// [`Self::decode_response_with_inventory`] for a filtered request.
    pub fn decode_response(self) -> Result<RobotRdnsList, RobotRdnsDecodeError> {
        if self.request.server_address().is_some() {
            return Err(RobotRdnsDecodeError::UnverifiableServerFilter);
        }
        self.decode_list()
    }

    /// Decodes non-empty filtered membership using checked Robot IP inventory.
    ///
    /// Every returned address must occur in `inventory` with the exact main
    /// server address retained by this request. Empty responses remain
    /// unverifiable because the provider does not echo the filter. This method
    /// proves membership only, not completeness or authoritative absence. The
    /// inventory should be obtained from a freshly checked Robot IP list
    /// response. Provider state may still change between those two reads.
    pub fn decode_response_with_inventory(
        self,
        inventory: &RobotIpList,
    ) -> Result<RobotRdnsFilteredMembership, RobotRdnsDecodeError> {
        let Self { request, inner } = self;
        let Some(server_address) = request.server_address() else {
            return Err(RobotRdnsDecodeError::UnverifiableServerFilter);
        };
        let result = inner.decode_owned_with_workspace(decode_robot_rdns_list)?;
        let result = RobotRdnsFilteredMembership::new(result)
            .ok_or(RobotRdnsDecodeError::UnverifiableServerFilter)?;
        let assignments = assignment_index(inventory, server_address)?;
        if !result
            .as_slice()
            .iter()
            .all(|entry| entry.with_address(|address| assignments.binary_search(&address).is_ok()))
        {
            return Err(RobotRdnsDecodeError::ResponseIdentityMismatch);
        }
        Ok(result)
    }

    fn decode_list(self) -> Result<RobotRdnsList, RobotRdnsDecodeError> {
        self.inner
            .decode_owned_with_workspace(decode_robot_rdns_list)
    }
}

fn assignment_index(
    inventory: &RobotIpList,
    server_address: &crate::robot::RobotIpAddress,
) -> Result<Vec<IpAddr>, RobotRdnsDecodeError> {
    let expected_server = server_address.with_addr(|address| address);
    let mut assignments = Vec::new();
    assignments
        .try_reserve_exact(inventory.len())
        .map_err(|_| RobotRdnsDecodeError::Allocation)?;
    for entry in inventory.as_slice() {
        if entry.with_server_address(|address| address == expected_server) {
            assignments.push(entry.with_address(|address| address));
        }
    }
    assignments.sort_unstable();
    Ok(assignments)
}

impl CheckedRobotRdns<'_, '_, RobotRdnsGetRequest> {
    /// Decodes and binds one entry to the request address.
    pub fn decode_response(self) -> Result<RobotRdns, RobotRdnsDecodeError> {
        decode_bound(self.request.address(), self.inner)
    }
}

macro_rules! mutation_decoder {
    ($request:ty) => {
        impl CheckedRobotRdns<'_, '_, $request> {
            /// Requires the provider to echo the exact requested address and PTR target.
            pub fn decode_response(self) -> Result<RobotRdns, RobotRdnsDecodeError> {
                let result = decode_bound(self.request.address(), self.inner)?;
                if result.ptr() == self.request.ptr() {
                    Ok(result)
                } else {
                    Err(RobotRdnsDecodeError::MutationOutcomeMismatch)
                }
            }
        }
    };
}

mutation_decoder!(RobotRdnsSetRequest);
mutation_decoder!(RobotRdnsUpdateRequest);

impl CheckedRobotRdns<'_, '_, RobotRdnsDeleteRequest> {
    /// Accepts and clears the exact empty delete acknowledgement.
    pub fn decode_response(self) -> Result<(), RobotRdnsDecodeError> {
        drop(self);
        Ok(())
    }
}

fn decode_bound(
    expected: &crate::robot::RobotIpAddress,
    checked: CheckedResponseGuard<'_>,
) -> Result<RobotRdns, RobotRdnsDecodeError> {
    checked.decode_owned_with_workspace(|response, workspace| {
        decode_robot_rdns(response, expected, workspace)
    })
}

#[cfg(doctest)]
mod compile_fail {
    /// Different operation types cannot consume each other's checked response.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::{
    ///     CheckedRobotRdns, RobotRdnsGetRequest, RobotRdnsUpdateRequest,
    /// };
    /// fn consume(_: CheckedRobotRdns<'_, '_, RobotRdnsGetRequest>) {}
    /// fn wrong(response: CheckedRobotRdns<'_, '_, RobotRdnsUpdateRequest>) {
    ///     consume(response);
    /// }
    /// ```
    fn association() {}
}
