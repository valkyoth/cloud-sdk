//! Builds a validated DNS Zone create endpoint without executing the mutation.

use cloud_sdk_hetzner::dns::zones::{ZoneCreateMode, ZoneCreateRequest, ZoneName, ZoneTtl};

fn main() {
    let Ok(name) = ZoneName::new("example.com") else {
        return;
    };
    let Ok(ttl) = ZoneTtl::new(3_600) else {
        return;
    };
    let request = ZoneCreateRequest::new(name, ZoneCreateMode::Primary);
    let request = request.with_ttl(ttl);
    let endpoint = request.endpoint();
    let mut path = [0_u8; 16];
    let Ok(written) = endpoint.write_path(&mut path) else {
        return;
    };

    assert_eq!(endpoint.method().as_str(), "POST");
    assert_eq!(request.ttl().map(ZoneTtl::get), Some(3_600));
    assert_eq!(path.get(..written), Some("/zones".as_bytes()));
}
