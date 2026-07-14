//! Builds a provider-neutral transport request without performing I/O.

use cloud_sdk::Method;
use cloud_sdk::transport::{RequestTarget, TransportRequest};

fn main() {
    let Ok(target) = RequestTarget::new("/resources?page=1") else {
        return;
    };
    let request = TransportRequest::new(Method::Get, target);

    assert_eq!(request.method(), Method::Get);
    assert_eq!(request.target().as_str(), "/resources?page=1");
    assert!(request.body().is_empty());
}
