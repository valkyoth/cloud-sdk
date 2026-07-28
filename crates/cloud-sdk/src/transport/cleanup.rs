//! Mandatory response cleanup and optional platform hooks.

/// Optional additive cleanup hook for caller-owned response storage.
///
/// Core volatile-clears the complete storage before and after this hook. The
/// hook can add platform controls such as cache eviction, but a no-op or faulty
/// implementation cannot replace or weaken the mandatory clear.
pub trait ResponseStorageSanitizer {
    /// Applies an additive platform-specific cleanup operation.
    fn sanitize_response_storage(&self, response_storage: &mut [u8]);
}

pub(crate) fn sanitize_response_storage(
    storage: &mut [u8],
    additive: Option<&dyn ResponseStorageSanitizer>,
) {
    cloud_sdk_sanitization::sanitize_bytes(storage);
    if let Some(additive) = additive {
        let mut final_clear = MandatoryFinalClear { storage };
        additive.sanitize_response_storage(final_clear.storage_mut());
    }
}

struct MandatoryFinalClear<'a> {
    storage: &'a mut [u8],
}

impl MandatoryFinalClear<'_> {
    fn storage_mut(&mut self) -> &mut [u8] {
        self.storage
    }
}

impl Drop for MandatoryFinalClear<'_> {
    fn drop(&mut self) {
        cloud_sdk_sanitization::sanitize_bytes(self.storage);
    }
}
