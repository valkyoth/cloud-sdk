use cloud_sdk::rate_limit::RateLimit;
use cloud_sdk::transport::{
    ResponseContentType, ResponseHeaders, StatusCode, TransportResponse,
};

fn main() {
    let external = b"unadmitted";
    let _ = TransportResponse {
        status: StatusCode::OK,
        body: external,
        content_type: None::<ResponseContentType>,
        rate_limit: None::<RateLimit>,
        headers: ResponseHeaders::new(),
    };
}
