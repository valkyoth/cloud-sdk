#![no_main]

use cloud_sdk::pagination::{
    NumberedPageMetadata, NumberedPagination, PageNumber, PaginationBudget, PaginationLimits,
    SnapshotPolicy,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let first = page(data.first().copied().unwrap_or(0));
    let per_page = u64::from(data.get(1).copied().unwrap_or(0)).saturating_add(1);
    let limits = PaginationLimits::new(
        u32::from(data.get(2).copied().unwrap_or(0) % 16) + 1,
        16_384,
        256,
    );
    let (Ok(first), Ok(limits)) = (first, limits) else {
        return;
    };
    let budget = PaginationBudget::new(limits, SnapshotPolicy::Forbidden);
    let Ok(mut cursor) = NumberedPagination::new(first, per_page, budget) else {
        return;
    };

    for chunk in data.get(3..).unwrap_or_default().chunks(7).take(64) {
        let current = page(chunk.first().copied().unwrap_or(0));
        let flags = chunk.get(1).copied().unwrap_or(0);
        let previous = optional_page(chunk.get(2).copied(), flags & 1 != 0);
        let next = optional_page(chunk.get(3).copied(), flags & 2 != 0);
        let last = optional_page(chunk.get(4).copied(), flags & 4 != 0);
        let total = (flags & 8 != 0).then(|| u64::from(chunk.get(5).copied().unwrap_or(0)));
        let entries = usize::from(chunk.get(6).copied().unwrap_or(0));
        let Ok(current) = current else {
            continue;
        };
        let (Ok(previous), Ok(next), Ok(last)) = (previous, next, last) else {
            continue;
        };
        let Ok(metadata) =
            NumberedPageMetadata::new(current, per_page, previous, next, last, total)
        else {
            continue;
        };

        let before_page = cursor.next_page();
        let before_progress = cursor.progress();
        let result = cursor.observe(metadata, entries, None, None);
        if result.is_err() {
            assert_eq!(cursor.next_page(), before_page);
            assert_eq!(cursor.progress(), before_progress);
        }
    }
});

fn page(value: u8) -> Result<PageNumber, cloud_sdk::pagination::PaginationError> {
    PageNumber::new(u64::from(value))
}

fn optional_page(
    value: Option<u8>,
    present: bool,
) -> Result<Option<PageNumber>, cloud_sdk::pagination::PaginationError> {
    if present {
        value.map(page).transpose()
    } else {
        Ok(None)
    }
}
