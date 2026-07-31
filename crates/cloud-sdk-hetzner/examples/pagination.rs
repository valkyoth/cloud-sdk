//! Decodes and validates one Hetzner pagination envelope.

use cloud_sdk::pagination::{
    NumberedPagination, PageNumber, PaginationBudget, PaginationLimits, SnapshotPolicy,
};
use cloud_sdk_hetzner::serde::PaginationEnvelope;

fn main() {
    let body = br#"{
        "servers": [{"id": 42}],
        "meta": {"pagination": {
            "page": 1,
            "per_page": 25,
            "previous_page": null,
            "next_page": null,
            "last_page": 1,
            "total_entries": 1
        }}
    }"#;
    let Ok(envelope) = serde_json::from_slice::<PaginationEnvelope>(body) else {
        return;
    };
    let metadata = envelope.pagination();
    let Ok(first) = PageNumber::new(1) else {
        return;
    };
    let Ok(limits) = PaginationLimits::new(2, 50, 128) else {
        return;
    };
    let budget = PaginationBudget::new(limits, SnapshotPolicy::Forbidden);
    let Ok(mut pagination) =
        NumberedPagination::new(first, u64::from(metadata.per_page().get()), budget)
    else {
        return;
    };
    let Ok(boundary) = pagination.observe(metadata.as_core(), 1, None, None) else {
        return;
    };

    assert!(boundary.is_terminal());
    assert_eq!(metadata.total_entries(), Some(1));
}
