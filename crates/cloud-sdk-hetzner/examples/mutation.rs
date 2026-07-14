//! Builds a validated server-create endpoint without executing the mutation.

use cloud_sdk_hetzner::cloud::servers::{ServerCreateRequest, ServerName, ServerReference};

fn main() {
    let Ok(name) = ServerName::new("web-1") else {
        return;
    };
    let Ok(server_type) = ServerReference::new("cpx22") else {
        return;
    };
    let Ok(image) = ServerReference::new("ubuntu-24.04") else {
        return;
    };
    let Ok(request) = ServerCreateRequest::try_new(Some(name), Some(server_type), Some(image))
    else {
        return;
    };
    let endpoint = request.endpoint();
    let mut path = [0_u8; 16];
    let Ok(written) = endpoint.write_path(&mut path) else {
        return;
    };

    assert_eq!(endpoint.method().as_str(), "POST");
    assert_eq!(path.get(..written), Some("/servers".as_bytes()));
}
