#![no_main]

use cloud_sdk_hetzner::metadata::{MetadataRoute, decode_metadata_body};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((&selector, body)) = data.split_first() else {
        return;
    };
    let route = match selector {
        b's' => MetadataRoute::Summary,
        b'h' => MetadataRoute::Hostname,
        b'i' => MetadataRoute::InstanceId,
        b'4' => MetadataRoute::PublicIpv4,
        b'n' => MetadataRoute::PrivateNetworks,
        b'a' => MetadataRoute::AvailabilityZone,
        b'r' => MetadataRoute::Region,
        _ => return,
    };
    let _ = decode_metadata_body(route, body);
});
