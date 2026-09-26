use cloud_sdk::{Method, transport::*};
use cloud_sdk_cratesio::endpoint::OfficialCratesIoEndpoint;

pub const METADATA: &str = "/api/v1/site_metadata";
pub const KEYWORDS: &str = "/api/v1/keywords?per_page=1";
pub const FOLLOWING: &str = "/api/v1/crates?following=yes&per_page=1";

pub fn check(request: TransportRequest<'_>, authorized: bool) -> Result<(), &'static str> {
    let allowed = if authorized {
        request.target().as_str() == FOLLOWING
    } else {
        matches!(request.target().as_str(), METADATA | KEYWORDS)
    };
    if request.method() != Method::Get
        || !allowed
        || !request.body().is_empty()
        || request.headers().get("authorization").is_some()
        || request.headers().get("cookie").is_some()
    {
        return Err("live request is not an admitted read");
    }
    Ok(())
}

pub struct ReadOnly<T>(pub T);
impl<T: BoundTransport> BoundTransport for ReadOnly<T> {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.0.endpoint_identity()
    }
}
impl<T: BoundUserAgent> BoundUserAgent for ReadOnly<T> {
    fn configured_user_agent(&self) -> &[u8] {
        self.0.configured_user_agent()
    }
}
impl<T: BlockingRawHttpExecutor + BoundTransport> BlockingRawHttpExecutor for ReadOnly<T> {
    type Error = &'static str;
    fn execute(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        check(request, false)?;
        OfficialCratesIoEndpoint::production_api()
            .verify_transport(&self.0)
            .map_err(|_| "live endpoint rejected")?;
        self.0
            .execute(request, policy, response)
            .map_err(|_| "live read transport failed")
    }
}
impl<T: BlockingAuthorizedRawHttpExecutor> BlockingAuthorizedRawHttpExecutor for ReadOnly<T> {
    fn execute_authorized(
        &self,
        expected: EndpointIdentity<'_>,
        authorization: HeaderValue<'_>,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        check(request, true)?;
        OfficialCratesIoEndpoint::production_api()
            .verify_transport(&self.0)
            .map_err(|_| "live endpoint rejected")?;
        self.0
            .execute_authorized(expected, authorization, request, policy, response)
            .map_err(|_| "live read transport failed")
    }
}
