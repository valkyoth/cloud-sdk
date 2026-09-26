use crate::{
    discovery::{DiscoveryError as Error, DiscoveryExecutionError as Failure},
    wire::CratesIoWireError,
};
use cloud_sdk::{
    rate_limit::{RetryAfter, WallClockTimestamp},
    transport::{HeaderName, MediaType, RawResponsePolicy, ResponseBuffer, ResponseMediaPolicy},
};

pub(crate) fn policy(maximum: usize) -> Result<RawResponsePolicy<'static>, Error> {
    RawResponsePolicy::new(
        0,
        maximum,
        ResponseMediaPolicy::Forbidden,
        ResponseMediaPolicy::Required(&[MediaType::JSON]),
        &[
            HeaderName::new("content-type").map_err(|_| Error::Value)?,
            HeaderName::new("retry-after").map_err(|_| Error::Value)?,
            HeaderName::new("content-encoding").map_err(|_| Error::Value)?,
        ],
        8,
    )
    .map_err(|_| Error::Value)
}
pub(crate) fn admit<E>(
    response: &ResponseBuffer<'_>,
    now: u64,
) -> Result<Option<RetryAfter>, Failure<E>> {
    response
        .with_response(|r| {
            if !r.body().is_empty() {
                return Err(CratesIoWireError::UnexpectedStatus);
            }
            if r.headers().get("content-type").is_some()
                || r.headers().get("content-encoding").is_some()
            {
                return Err(CratesIoWireError::ContentType);
            }
            r.headers()
                .get("retry-after")
                .map(|h| {
                    RetryAfter::parse(h.value(), WallClockTimestamp::new(now))
                        .map_err(|_| CratesIoWireError::RetryAfter)
                })
                .transpose()
        })
        .map_err(|_| Failure::Staging)?
        .map_err(Failure::Wire)
}
