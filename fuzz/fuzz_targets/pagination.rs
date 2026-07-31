#![no_main]

use cloud_sdk::pagination::{
    NumberedPageMetadata, NumberedPagination, OffsetPageMetadata, OffsetPagination, PageNumber,
    PaginationBudget, PaginationLimits, SnapshotPolicy,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    fuzz_numbered(data);
    fuzz_offset(data);
});

fn fuzz_numbered(data: &[u8]) {
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
}

fn fuzz_offset(data: &[u8]) {
    let page_size = u64::from(data.first().copied().unwrap_or(0)).saturating_add(1);
    let requests = u32::from(data.get(1).copied().unwrap_or(0) % 16).saturating_add(1);
    let Ok(limits) = PaginationLimits::new(requests, 16_384, 256) else {
        return;
    };
    let budget = PaginationBudget::new(limits, SnapshotPolicy::Forbidden);
    let Ok(mut pagination) = OffsetPagination::new(0, page_size, budget) else {
        return;
    };

    for chunk in data.get(2..).unwrap_or_default().chunks(27).take(64) {
        let offset = read_u64(chunk, 0);
        let next = (chunk.get(24).copied().unwrap_or(0) & 1 != 0).then(|| read_u64(chunk, 8));
        let total = (chunk.get(24).copied().unwrap_or(0) & 2 != 0).then(|| read_u64(chunk, 16));
        let entries = usize::from(chunk.get(25).copied().unwrap_or(0));
        let response_page_size = if chunk.get(26).copied().unwrap_or(0) & 1 == 0 {
            page_size
        } else {
            page_size.saturating_add(1)
        };
        let Ok(metadata) = OffsetPageMetadata::new(offset, response_page_size, next, total) else {
            continue;
        };
        let before_offset = pagination.next_offset();
        let before_progress = pagination.progress();
        if pagination.observe(metadata, entries, None, None).is_err() {
            assert_eq!(pagination.next_offset(), before_offset);
            assert_eq!(pagination.progress(), before_progress);
        }
    }
}

fn read_u64(data: &[u8], start: usize) -> u64 {
    let Some(end) = start.checked_add(8) else {
        return 0;
    };
    let Some(bytes) = data.get(start..end) else {
        return 0;
    };
    let Ok(bytes) = <[u8; 8]>::try_from(bytes) else {
        return 0;
    };
    u64::from_be_bytes(bytes)
}

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
