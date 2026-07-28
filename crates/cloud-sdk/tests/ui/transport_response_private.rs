use cloud_sdk::transport::{StatusCode, TransportResponse};

fn main() {
    let external = b"unadmitted";
    let _ = TransportResponse {
        status: StatusCode::OK,
        body: external,
        metadata: unreachable!(),
    };
}
