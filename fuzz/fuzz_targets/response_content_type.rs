#![no_main]

use cloud_sdk::transport::{ContentType, MediaType, ResponseContentType};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = core::str::from_utf8(data) else {
        return;
    };

    if let Ok(content_type) = ContentType::new(text) {
        assert_eq!(content_type.as_str(), text);
        assert!(!content_type.essence().is_empty());
        assert!(MediaType::new(content_type.essence()).is_ok());
    }

    if let Ok(media_type) = MediaType::new(text) {
        assert_eq!(media_type.as_str(), text);
        assert!(ContentType::new(text).is_ok());
    }

    if let Ok(response) = ResponseContentType::new(text) {
        assert_eq!(response.as_str(), text);
        assert_eq!(response.as_content_type().as_str(), text);
        let essence = response.as_content_type().essence();
        if let Ok(media_type) = MediaType::new(essence) {
            assert!(response.matches(media_type));
        }
    }
});
