use super::*;
use cloud_sdk::{Method, authentication::*, operation::OperationId, pagination::*, transport::*};
use core::cell::Cell;
use std::string::ToString;

struct Compare {
    endpoint: HttpsEndpoint,
    calls: Cell<usize>,
    admitted: bool,
}
impl BoundTransport for Compare {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.endpoint.identity()
    }
}
impl BlockingAuthenticatedTransport for Compare {
    type Error = ();
    fn send_authenticated(
        &self,
        request: AuthenticatedRequest<'_, '_>,
        _: &mut ResponseWriter<'_>,
    ) -> Result<(), ()> {
        self.calls.set(self.calls.get().checked_add(1).ok_or(())?);
        let target = request.transport_request().target();
        let old = self.endpoint.compose(target);
        let new = compose(&self.endpoint, target);
        assert_eq!(old.is_ok(), self.admitted);
        assert_eq!(new.is_ok(), self.admitted);
        match (old, new) {
            (Ok(old), Ok(new)) => assert_eq!(old.as_str(), new.to_string()),
            (Err(_), Err(RawHttpError::TargetRejected)) => {}
            _ => unreachable!("composition mismatch"),
        }
        Ok(())
    }
}

#[test]
fn raw_uri_preserves_provider_link_query_grammar_and_normalization_rejections() {
    for (value, admitted) in [
        ("/search?q=a+b==&q=%41&x=%2f&raw=?:/@!$();,", true),
        ("/search?", true),
        ("/search?q=a'b", false),
        ("/search?q=a%27b", true),
    ] {
        let endpoint = HttpsEndpoint::new_custom(
            "https://example.test",
            CustomEndpointAcknowledgement::trusted_operator_configuration(),
        )
        .unwrap_or_else(|_| unreachable!("endpoint"));
        let operation =
            OperationId::new("list_fixture").unwrap_or_else(|_| unreachable!("operation"));
        let binding = ProviderLinkBinding::new(
            endpoint
                .identity()
                .unwrap_or_else(|_| unreachable!("identity")),
            operation,
            RequestPath::new("/search").unwrap_or_else(|_| unreachable!("path")),
        );
        let mut source = value.as_bytes().to_vec();
        let mut storage = [0; 256];
        let link = ValidatedProviderLink::transfer_from(
            &mut source,
            &mut storage,
            binding,
            PaginationLimits::new(1, 1, 256).unwrap_or_else(|_| unreachable!("limits")),
        )
        .unwrap_or_else(|_| unreachable!("valid link fixture"));
        let fixture = Compare {
            endpoint: endpoint.clone(),
            calls: Cell::new(0),
            admitted,
        };
        let policy = RawResponsePolicy::new(
            0,
            0,
            ResponseMediaPolicy::Forbidden,
            ResponseMediaPolicy::Forbidden,
            &[],
            0,
        )
        .unwrap_or_else(|_| unreachable!("policy"));
        let authentication = AuthenticationScopePolicy::new(
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
        );
        let mut body = [];
        let mut headers = [];
        let mut response = ResponseBuffer::new(&mut body, 0, &mut headers);
        assert!(
            link.execute_blocking(
                &fixture,
                Method::Get,
                operation,
                authentication,
                policy,
                response.writer()
            )
            .is_ok()
        );
        assert_eq!(fixture.calls.get(), 1);
    }
}
