use super::*;
use crate::query::*;
use cloud_sdk::{
    Method,
    authentication::{
        AuthenticatedRequest, AuthenticationScopePolicy, BlockingAuthenticatedTransport,
        ScopeRequirement,
    },
    operation::OperationId,
    transport::{
        BoundTransport, EndpointIdentity, EndpointIdentityError, RawResponsePolicy, ResponseBuffer,
        ResponseMediaPolicy, ResponseWriter,
    },
};
use core::cell::Cell;

fn valid<T, E>(v: Result<T, E>) -> T {
    v.unwrap_or_else(|_| unreachable!("execution fixture"))
}

struct Transport {
    endpoint: OfficialCratesIoEndpoint,
    calls: Cell<u32>,
}
impl BoundTransport for Transport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(valid(self.endpoint.identity()))
    }
}
impl BlockingAuthenticatedTransport for Transport {
    type Error = ();
    fn send_authenticated(
        &self,
        request: AuthenticatedRequest<'_, '_>,
        _: &mut ResponseWriter<'_>,
    ) -> Result<(), ()> {
        assert_eq!(
            request.transport_request().target().as_str(),
            "/api/v1/crates?page=2"
        );
        self.calls.set(self.calls.get().saturating_add(1));
        Err(())
    }
}

#[test]
fn provider_link_transfer_retains_dispatch_binding_and_clears_on_drop() {
    let endpoint = OfficialCratesIoEndpoint::production_api();
    let parts = [PathSegment::Fixed(FixedSegment::Crates)];
    let link = valid(PageLink::new(
        endpoint,
        valid(ApiPath::new(&parts)),
        valid(Query::new(QueryOperation::Crates, &[])),
        "?page=2",
        Direction::Next,
    ));
    let mut path = [0; 128];
    let mut storage = [0xA5; 128];
    let limits = valid(PaginationLimits::new(2, 100, 128));
    let auth = AuthenticationScopePolicy::new(
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let policy = valid(RawResponsePolicy::new(
        0,
        0,
        ResponseMediaPolicy::Forbidden,
        ResponseMediaPolicy::Forbidden,
        &[],
        0,
    ));
    let mut body = [];
    let mut headers = [];
    let mut response = ResponseBuffer::new(&mut body, 0, &mut headers);
    let transport = Transport {
        endpoint,
        calls: Cell::new(0),
    };
    {
        let transferred = valid(link.transfer_to(&mut path, &mut storage, limits));
        for (endpoint, method, operation) in [
            (
                OfficialCratesIoEndpoint::staging_api(),
                Method::Get,
                link.operation(),
            ),
            (endpoint, Method::Post, link.operation()),
            (endpoint, Method::Get, valid(OperationId::new("other"))),
        ] {
            let transport = Transport {
                endpoint,
                calls: Cell::new(0),
            };
            assert!(
                transferred
                    .execute_blocking(
                        &transport,
                        method,
                        operation,
                        auth,
                        policy,
                        response.writer()
                    )
                    .is_err()
            );
            assert_eq!(transport.calls.get(), 0);
        }
        assert!(
            transferred
                .execute_blocking(
                    &transport,
                    Method::Get,
                    link.operation(),
                    auth,
                    policy,
                    response.writer()
                )
                .is_err()
        );
        assert_eq!(transport.calls.get(), 1);
    }
    assert_eq!(storage, [0; 128]);
    for capacity in 0.."/api/v1/crates?page=2".len() {
        let mut path = [0; 128];
        let mut storage = [0xA5; 128];
        let output = storage
            .get_mut(..capacity)
            .unwrap_or_else(|| unreachable!("capacity"));
        assert!(link.transfer_to(&mut path, output, limits).is_err());
        assert!(output.iter().all(|b| *b == 0));
    }
}
