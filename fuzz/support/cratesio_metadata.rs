use cloud_sdk::transport::StreamLimits;
use cloud_sdk_cratesio::publishing::{PublishMetadata, PublishRequest};

pub fn exercise(data: &[u8]) -> bool {
    let Ok(metadata) = PublishMetadata::from_json(data) else {
        return false;
    };
    let independent: serde_json::Value = serde_json::from_slice(data).expect("accepted JSON");
    assert!(independent.is_object());
    assert_eq!(format!("{metadata:?}"), "PublishMetadata([redacted])");
    for archive_length in [0, 1, 4096, 536_870_912, 536_870_913, u64::MAX] {
        let metadata = PublishMetadata::from_json(data).expect("repeat parsing");
        let limits =
            StreamLimits::new(600_000_000, 4096, 200_000, 200_000, 2).expect("fixed limits");
        let result = PublishRequest::new(metadata, archive_length, limits);
        if archive_length == 0 || archive_length > 536_870_912 {
            assert!(result.is_err());
        } else {
            let request = result.expect("bounded framing");
            assert_eq!(
                request.content_length(),
                data.len() as u64 + archive_length + 8
            );
            assert_eq!(request.archive_length(), archive_length);
            assert!(!request.permits_automatic_retry());
        }
    }
    true
}
