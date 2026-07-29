use cloud_sdk::transport::{ResponseBuffer, ResponseMetadata, StatusCode};

fn main() {
    let mut body = [0_u8; 8];
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut body, 8, &mut headers);
    let writer = response.writer();
    let _ = writer.body_mut();
    let _ = writer.headers_mut();
    let _ = writer.commit(StatusCode::OK, 0, ResponseMetadata::EMPTY);
}
