use super::guard::{self, ReadOnly};
use cloud_sdk::{Method, transport::*};
use std::cell::Cell;

struct Recorder {
    host: &'static str,
    calls: Cell<usize>,
}
impl BoundTransport for Recorder {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        EndpointIdentity::new(EndpointScheme::Https, self.host, 443, "/")
    }
}
impl BlockingRawHttpExecutor for Recorder {
    type Error = ();
    fn execute(
        &self,
        _: TransportRequest<'_>,
        _: RawResponsePolicy<'_>,
        _: &mut ResponseWriter<'_>,
    ) -> Result<(), ()> {
        self.calls.set(self.calls.get().saturating_add(1));
        Err(())
    }
}
impl BlockingAuthorizedRawHttpExecutor for Recorder {
    fn execute_authorized(
        &self,
        _: EndpointIdentity<'_>,
        _: HeaderValue<'_>,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        output: &mut ResponseWriter<'_>,
    ) -> Result<(), ()> {
        self.execute(request, policy, output)
    }
}

#[test]
fn rejected_requests_never_dispatch_and_admitted_reads_dispatch_once()
-> Result<(), Box<dyn std::error::Error>> {
    let policy = RawResponsePolicy::new(
        0,
        0,
        ResponseMediaPolicy::Forbidden,
        ResponseMediaPolicy::Forbidden,
        &[],
        0,
    )?;
    for host in ["crates.io", "staging.crates.io", "wrong.invalid"] {
        for authorized in [false, true] {
            for path in [guard::METADATA, guard::FOLLOWING, "/api/v1/crates/new"] {
                for method in [Method::Get, Method::Put, Method::Delete, Method::Patch] {
                    let transport = ReadOnly(Recorder {
                        host,
                        calls: Cell::new(0),
                    });
                    let request = TransportRequest::new(method, RequestTarget::new(path)?);
                    let allowed = host == "crates.io" && guard::check(request, authorized).is_ok();
                    let mut body = [];
                    let mut headers = [];
                    let mut output = ResponseBuffer::new(&mut body, 0, &mut headers);
                    let result = if authorized {
                        transport.execute_authorized(
                            transport.endpoint_identity()?,
                            HeaderValue::new("fixture")?,
                            request,
                            policy,
                            output.writer(),
                        )
                    } else {
                        transport.execute(request, policy, output.writer())
                    };
                    assert!(result.is_err());
                    assert_eq!(transport.0.calls.get(), usize::from(allowed));
                }
            }
        }
    }
    Ok(())
}
