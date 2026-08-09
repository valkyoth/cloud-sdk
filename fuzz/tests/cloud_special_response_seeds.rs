use cloud_sdk_hetzner::serde::ApiErrorEnvelope;

const METRICS: &[u8] = include_bytes!("../seeds/cloud_special_responses/metrics.json");
const METRICS_NEGATIVE: &[u8] =
    include_bytes!("../seeds/cloud_special_responses/metrics-negative-underflow.json");
const METRICS_POSITIVE: &[u8] =
    include_bytes!("../seeds/cloud_special_responses/metrics-positive-underflow.json");
const ERROR_UNICODE: &[u8] =
    include_bytes!("../seeds/cloud_special_responses/error-unicode-control.json");
const DNS_ZONE: &[u8] = include_bytes!("../seeds/cloud_special_responses/dns-zone.json");
const DNS_RRSET: &[u8] = include_bytes!("../seeds/cloud_special_responses/dns-rrset.json");
const CERTIFICATE: &[u8] = include_bytes!("../seeds/cloud_special_responses/certificate.json");
const SSH_KEY: &[u8] = include_bytes!("../seeds/cloud_special_responses/ssh-key.json");
const STORAGE_BOX: &[u8] = include_bytes!("../seeds/cloud_special_responses/storage-box.json");
const STORAGE_BOX_TYPE: &[u8] =
    include_bytes!("../seeds/cloud_special_responses/storage-box-type.json");
const STORAGE_SNAPSHOT: &[u8] =
    include_bytes!("../seeds/cloud_special_responses/storage-snapshot.json");
const STORAGE_SUBACCOUNT: &[u8] =
    include_bytes!("../seeds/cloud_special_responses/storage-subaccount.json");

fn split(seed: &[u8]) -> (&[u8], &[u8]) {
    let (controls, payload) = seed.split_at(3);
    assert!(!payload.is_empty());
    assert!(serde_json::from_slice::<serde_json::Value>(payload).is_ok());
    (controls, payload)
}

#[test]
fn named_metrics_seeds_reach_the_checked_success_json_route() {
    for seed in [METRICS, METRICS_NEGATIVE, METRICS_POSITIVE] {
        let (controls, _) = split(seed);
        assert_eq!(controls[0] % 8, 1, "seed must select metrics");
        assert_eq!(controls[1] & 1, 0, "seed must select success status");
        assert_eq!(controls[2] % 3, 0, "seed must select JSON content type");
    }
}

#[test]
fn unicode_error_seed_reaches_both_error_code_decoders() {
    let (controls, payload) = split(ERROR_UNICODE);
    assert_eq!(controls[0] % 8, 1, "seed must select metrics operation");
    assert_ne!(controls[1] & 1, 0, "seed must select provider error status");
    assert_eq!(controls[2] % 3, 0, "seed must select JSON content type");
    assert!(serde_json::from_slice::<ApiErrorEnvelope<'_>>(payload).is_err());
}

#[test]
fn named_dns_seeds_reach_dedicated_checked_models() {
    for (seed, selector) in [(DNS_ZONE, 4), (DNS_RRSET, 5)] {
        let (controls, _) = split(seed);
        assert_eq!(controls[0] % 8, selector);
        assert_eq!(controls[1] & 1, 0, "seed must select success status");
        assert_eq!(controls[2] % 3, 0, "seed must select JSON content type");
    }
}

#[test]
fn named_security_seeds_reach_protected_checked_models() {
    for (seed, selector) in [(CERTIFICATE, 8), (SSH_KEY, 9)] {
        let (controls, _) = split(seed);
        assert_eq!(controls[0] % 10, selector);
        assert_eq!(controls[1] & 1, 0, "seed must select success status");
        assert_eq!(controls[2] % 3, 0, "seed must select JSON content type");
    }
}

#[test]
fn named_storage_seeds_reach_source_complete_checked_models() {
    for (seed, selector) in [
        (STORAGE_BOX, 10),
        (STORAGE_BOX_TYPE, 11),
        (STORAGE_SNAPSHOT, 12),
        (STORAGE_SUBACCOUNT, 13),
    ] {
        let (controls, _) = split(seed);
        assert_eq!(controls[0] % 14, selector);
        assert_eq!(controls[1] & 1, 0, "seed must select success status");
        assert_eq!(controls[2] % 3, 0, "seed must select JSON content type");
    }
}
