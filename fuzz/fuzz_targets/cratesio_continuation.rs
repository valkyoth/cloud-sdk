#![no_main]

#[path = "../support/cratesio_continuation.rs"]
mod support;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    support::exercise(data);
    // Also exercise text seed files without their terminal line delimiter.
    if let Some(line) = data.strip_suffix(b"\n") {
        support::exercise(line);
    }
});
