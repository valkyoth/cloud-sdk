#![no_main]

use cloud_sdk_hetzner::serde::{
    ActionEnvelope, ApiErrorEnvelope, PaginationEnvelope, ResponseBytes,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = ResponseBytes::new(data) else {
        return;
    };
    let bytes = input.as_slice();

    if let Ok(envelope) = serde_json::from_slice::<ActionEnvelope<'_>>(bytes) {
        let action = envelope.action();
        assert_ne!(action.id().get(), 0);
        assert!(!action.command().is_empty());
        assert!(action.progress() <= 100);
        assert!(!action.started().is_empty());
        assert!(
            action
                .resources()
                .iter()
                .all(|resource| resource.id().get() != 0)
        );
    }
    if let Ok(envelope) = serde_json::from_slice::<ApiErrorEnvelope<'_>>(bytes) {
        assert!(!envelope.error().message().is_empty());
    }
    if let Ok(envelope) = serde_json::from_slice::<PaginationEnvelope>(bytes) {
        let metadata = envelope.pagination();
        assert_ne!(metadata.page().get(), 0);
        assert_ne!(metadata.per_page().get(), 0);
    }
});
