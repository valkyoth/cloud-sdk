use cloud_sdk::transport::{
    BlockingRawHttpExecutor, BoundTransport, EndpointIdentity, EndpointIdentityError,
    HeaderSensitivity, RawResponsePolicy, ResponseBuffer, ResponseMetadata, ResponseWriter,
    StatusCode, TransportRequest,
};
use cloud_sdk_cratesio::endpoint::{
    ApiRequestTarget, OfficialCratesIoEndpoint, ProductionDownloadResponse,
};

const LOCATION: &str = "https://static.crates.io/crates/serde/serde-1.0.0.crate";
struct Source<'a>(&'a [u8]);
impl BoundTransport for Source<'_> {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(OfficialCratesIoEndpoint::production_api()
            .identity()
            .expect("fixed origin"))
    }
}
impl BlockingRawHttpExecutor for Source<'_> {
    type Error = ();
    fn execute(
        &self,
        request: TransportRequest<'_>,
        _: RawResponsePolicy<'_>,
        writer: &mut ResponseWriter<'_>,
    ) -> Result<(), ()> {
        assert_eq!(request.method(), cloud_sdk::Method::Get);
        assert!(request.headers().as_slice().is_empty());
        let mut attempt = writer.begin_attempt().map_err(|_| ())?;
        attempt
            .headers_mut()
            .map_err(|_| ())?
            .try_push("location", self.0, HeaderSensitivity::Public)
            .map_err(|_| ())?;
        attempt
            .commit(
                StatusCode::new(302).expect("status"),
                0,
                ResponseMetadata::EMPTY,
            )
            .map_err(|_| ())
    }
}

pub fn exercise(data: &[u8]) -> bool {
    let mut body = [0xa5; 1];
    let mut headers = [0xa5; 8192];
    let mut storage = [0xa5; 4096];
    let accepted;
    {
        let mut response = ResponseBuffer::new(&mut body, 0, &mut headers);
        let result = ProductionDownloadResponse::execute_blocking(
            &Source(data),
            ApiRequestTarget::new("/api/v1/crates/serde/1.0.0/download").expect("source"),
            &mut response,
            &mut storage,
        );
        accepted = result.is_ok();
        if accepted {
            assert_eq!(data, LOCATION.as_bytes());
        }
    }
    assert!(body.iter().chain(&headers).all(|b| *b == 0));
    if accepted {
        let target = b"/crates/serde/serde-1.0.0.crate";
        assert_eq!(&storage[..target.len()], target);
        assert!(storage[target.len()..].iter().all(|b| *b == 0));
    } else {
        assert!(storage.iter().all(|b| *b == 0));
    }
    accepted
}
