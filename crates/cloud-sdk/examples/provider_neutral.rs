//! Builds a provider-neutral transport request without performing I/O.

use cloud_sdk::transport::{RequestTarget, TransportRequest};
use cloud_sdk::{ApiFamily, Method, Provider};

fn main() {
    let provider = Provider::Hetzner;
    let family = ApiFamily::Cloud;
    let Ok(target) = RequestTarget::new("/servers?page=1") else {
        return;
    };
    let request = TransportRequest::new(Method::Get, target);

    assert_eq!(provider, Provider::Hetzner);
    assert_eq!(family, ApiFamily::Cloud);
    assert_eq!(request.method(), Method::Get);
    assert_eq!(request.target().as_str(), "/servers?page=1");
    assert!(request.body().is_empty());
}
