#![no_main]

#[path = "../support/cratesio_metadata.rs"]
mod support;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    support::exercise(data);
});
