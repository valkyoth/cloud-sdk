#![no_main]

use cloud_sdk::{Method, transport::RequestTarget};
use cloud_sdk_hetzner::{
    query::{QueryBuilder, QueryParam, write_percent_encoded_component},
    request::EndpointPath,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = core::str::from_utf8(data) else {
        return;
    };
    let mut fields = text.splitn(7, '\n');
    let path = fields.next().unwrap_or_default();
    let key_a = fields.next().unwrap_or_default();
    let value_a = fields.next().unwrap_or_default();
    let key_b = fields.next().unwrap_or_default();
    let value_b = fields.next().unwrap_or_default();

    if let Ok(path) = EndpointPath::new(path) {
        assert!(path.as_str().starts_with('/'));
        assert!(RequestTarget::new(path.as_str()).is_ok());
    }
    if let Ok(target) = RequestTarget::new(path) {
        let request = cloud_sdk::transport::TransportRequest::new(Method::Get, target);
        assert_eq!(request.target().as_str(), path);
    }

    let mut builder = QueryBuilder::<4>::new();
    for (key, value) in [(key_a, value_a), (key_b, value_b)] {
        if let Ok(parameter) = QueryParam::new(key, value) {
            let _ = builder.push(parameter);
        }
    }

    let capacity = usize::from(data.first().copied().unwrap_or(0)).saturating_mul(64);
    let mut output = vec![0_u8; capacity.min(16_384)];
    if let Ok(len) = builder.write_percent_encoded(&mut output) {
        assert!(len <= output.len());
        assert!(output[..len].iter().all(u8::is_ascii));
    }

    let mut component = vec![0_u8; value_a.len().saturating_mul(3)];
    let encoded = write_percent_encoded_component(value_a, &mut component);
    assert!(encoded.is_ok());
    if let Ok(len) = encoded {
        assert!(component[..len].iter().all(u8::is_ascii));
    }
});
