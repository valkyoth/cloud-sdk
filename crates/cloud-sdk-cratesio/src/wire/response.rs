use core::fmt;

use cloud_sdk::incremental_json::{
    IncrementalJsonDecoder, IncrementalJsonError, IncrementalJsonLimits, IncrementalJsonProgress,
    IncrementalJsonVisitor,
};
use cloud_sdk::rate_limit::{RetryAfter, WallClockTimestamp};
use cloud_sdk::transport::{MediaType, ResponseBuffer, StatusCode, TransportResponse};

use super::envelope::EnvelopeVisitor;
use super::{CratesIoWireError, ProviderError};

/// SDK hard ceiling for JSON responses; callers should choose a smaller budget.
pub const MAX_JSON_RESPONSE_BYTES: usize = 8_388_608;

/// Exact operation success status and bounded JSON response contract.
///
/// The operation-specific client supplies the expected status. This is not an
/// empty-body or download policy; those workflows use separate contracts.
#[derive(Clone, Copy, Debug)]
pub struct JsonResponsePolicy {
    status: StatusCode,
    maximum: usize,
}

impl JsonResponsePolicy {
    /// Chooses an exact body-bearing 2xx status and a nonzero lowered byte bound.
    pub fn new(status: StatusCode, maximum: usize) -> Result<Self, CratesIoWireError> {
        if !status.is_success()
            || matches!(status.get(), 204 | 205)
            || maximum == 0
            || maximum > MAX_JSON_RESPONSE_BYTES
        {
            return Err(CratesIoWireError::InvalidPolicy);
        }
        Ok(Self { status, maximum })
    }

    /// Returns the response-writer budget to apply before reading from transport.
    #[must_use]
    pub const fn maximum_bytes(self) -> usize {
        self.maximum
    }

    /// Consumes one committed response, checks every policy, and either returns
    /// a cleanup-owning JSON success envelope or a payload-free error.
    ///
    /// The full document is validated before success is exposed, including all
    /// unknown fields and duplicate keys. Error text never leaves this boundary.
    pub fn admit<'a>(
        self,
        response: ResponseBuffer<'a>,
        now: WallClockTimestamp,
    ) -> Result<JsonSuccess<'a>, CratesIoWireError> {
        let retry_after = response
            .with_response(|wire| self.check(wire, now))
            .map_err(|_| CratesIoWireError::Uncommitted)??;
        Ok(JsonSuccess {
            response,
            policy: self,
            retry_after,
        })
    }

    fn check(
        self,
        response: TransportResponse<'_, '_>,
        now: WallClockTimestamp,
    ) -> Result<Option<RetryAfter>, CratesIoWireError> {
        if response.body().len() > self.maximum {
            return Err(CratesIoWireError::ResponseTooLarge);
        }
        check_content_type(response)?;
        let retry_after = response
            .headers()
            .get("retry-after")
            .map(|header| {
                RetryAfter::parse(header.value(), now).map_err(|_| CratesIoWireError::RetryAfter)
            })
            .transpose()?;
        let mut visitor = EnvelopeVisitor::default();
        let mut decoder = IncrementalJsonDecoder::with_limits(self.limits()?);
        decoder
            .push(response.body(), &mut visitor)
            .map_err(map_json_error)?;
        if decoder.finish(&mut visitor).map_err(map_json_error)?
            != IncrementalJsonProgress::Complete
        {
            return Err(CratesIoWireError::Json);
        }
        if response.status().is_error()
            || (response.status().is_success() && visitor.errors_present)
        {
            return Err(CratesIoWireError::Provider(ProviderError {
                status: response.status(),
                count: visitor.error_count,
                retry_after,
            }));
        }
        if response.status() != self.status {
            return Err(CratesIoWireError::UnexpectedStatus);
        }
        Ok(retry_after)
    }

    fn limits(self) -> Result<IncrementalJsonLimits, CratesIoWireError> {
        IncrementalJsonLimits::DEFAULT
            .with_input_bytes(self.maximum)
            .map_err(|_| CratesIoWireError::InvalidPolicy)
    }
}

fn map_json_error(error: IncrementalJsonError<CratesIoWireError>) -> CratesIoWireError {
    error
        .into_visitor_error()
        .unwrap_or(CratesIoWireError::Json)
}

fn check_content_type(response: TransportResponse<'_, '_>) -> Result<(), CratesIoWireError> {
    let content_type = response
        .content_type()
        .map_err(|_| CratesIoWireError::ContentType)?
        .ok_or(CratesIoWireError::ContentType)?;
    if !content_type.matches(MediaType::JSON) {
        return Err(CratesIoWireError::ContentType);
    }
    if let Some((_, parameter)) = content_type.as_str().split_once(';') {
        let (name, value) = parameter
            .trim_start_matches([' ', '\t'])
            .split_once('=')
            .ok_or(CratesIoWireError::ContentType)?;
        if !name.eq_ignore_ascii_case("charset")
            || !(value.eq_ignore_ascii_case("utf-8") || value.eq_ignore_ascii_case("\"utf-8\""))
        {
            return Err(CratesIoWireError::ContentType);
        }
    }
    if response
        .headers()
        .get("content-encoding")
        .is_some_and(|header| !header.value().eq_ignore_ascii_case(b"identity"))
    {
        return Err(CratesIoWireError::ContentType);
    }
    Ok(())
}

/// Fully validated JSON success envelope retaining mandatory response cleanup.
///
/// Resource-specific decoding remains in the numbered operation checkpoints.
/// This wrapper is deliberately not Clone and never lends raw wire bytes.
///
/// ```compile_fail
/// use cloud_sdk_cratesio::wire::JsonSuccess;
/// let forged = JsonSuccess {};
/// ```
pub struct JsonSuccess<'a> {
    response: ResponseBuffer<'a>,
    policy: JsonResponsePolicy,
    retry_after: Option<RetryAfter>,
}

impl JsonSuccess<'_> {
    /// Returns validated advisory delay metadata, never a retry instruction.
    #[must_use]
    pub const fn retry_after(&self) -> Option<RetryAfter> {
        self.retry_after
    }

    /// Replays validated JSON events to a trusted resource-model decoder, then
    /// clears the response on every exit. A stopped visitor is not completion.
    /// Visitors own any copies they retain and must protect sensitive outputs.
    pub fn visit<V: IncrementalJsonVisitor>(
        self,
        visitor: &mut V,
    ) -> Result<IncrementalJsonProgress, IncrementalJsonError<V::Error>> {
        let limits = self
            .policy
            .limits()
            .map_err(|_| IncrementalJsonError::InputLimit)?;
        self.response
            .with_response(|response| {
                let mut decoder = IncrementalJsonDecoder::with_limits(limits);
                if decoder.push(response.body(), visitor)? == IncrementalJsonProgress::Stopped {
                    return Ok(IncrementalJsonProgress::Stopped);
                }
                decoder.finish(visitor)
            })
            .map_err(|_| IncrementalJsonError::TerminalState)?
    }
}

impl fmt::Debug for JsonSuccess<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JsonSuccess([redacted])")
    }
}
