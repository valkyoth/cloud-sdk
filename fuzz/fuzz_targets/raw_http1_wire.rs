#![no_main]

use cloud_sdk_reqwest::fuzz_raw_http1_wire;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    fuzz_raw_http1_wire(data);
});
