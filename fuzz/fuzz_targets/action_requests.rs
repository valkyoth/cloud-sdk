#![no_main]

use cloud_sdk_hetzner::{
    actions::{ActionEndpoint, ActionId, ActionListRequest, ActionStatus, MAX_ACTION_ID},
    pagination::{Page, PerPage, SortDirection},
    security::certificates::{
        CertificateActionEndpoint, CertificateActionListForCertificateRequest,
        CertificateActionListRequest, CertificateActionSortField, CertificateId,
    },
};
use libfuzzer_sys::fuzz_target;

const MAX_OUTPUT_BYTES: usize = 16_384;

fuzz_target!(|data: &[u8]| {
    let capacity = output_capacity(data);
    let ids = action_ids(data);
    let statuses = action_statuses(data);
    let sorts = action_sorts(data);

    exercise_global_actions(&ids, capacity);

    let certificate_value = read_u64(data.get(2..).unwrap_or_default()).max(1);
    let Some(certificate_id) = CertificateId::new(certificate_value) else {
        return;
    };
    exercise_certificate_actions(certificate_id, &ids, &statuses, &sorts, capacity, data);
});

fn exercise_global_actions(ids: &[ActionId], capacity: usize) {
    check_path(ActionEndpoint::List, capacity);
    for id in ids.iter().copied().take(4) {
        check_path(ActionEndpoint::Get(id), capacity);
    }
    if let Ok(request) = ActionListRequest::try_new(ids) {
        let mut output = vec![0xa5; capacity];
        check_written(request.write_query(&mut output), &output, false);
    }
}

fn exercise_certificate_actions(
    certificate_id: CertificateId,
    ids: &[ActionId],
    statuses: &[ActionStatus],
    sorts: &[(CertificateActionSortField, SortDirection)],
    capacity: usize,
    data: &[u8],
) {
    check_certificate_path(CertificateActionEndpoint::ListAll, capacity);
    check_certificate_path(
        CertificateActionEndpoint::ListForCertificate(certificate_id),
        capacity,
    );
    for id in ids.iter().copied().take(4) {
        check_certificate_path(CertificateActionEndpoint::Get(id), capacity);
    }

    let page = Page::new(read_u64(data.get(10..).unwrap_or_default()).max(1));
    let per_page = PerPage::new(u16::from(data.get(18).copied().unwrap_or(0) % 50) + 1);
    let (Ok(page), Ok(per_page)) = (page, per_page) else {
        return;
    };

    let global = CertificateActionListRequest::new()
        .with_action_ids(ids)
        .and_then(|request| request.with_statuses(statuses))
        .and_then(|request| request.with_sorts(sorts));
    if let Ok(request) = global {
        let mut output = vec![0xa5; capacity];
        check_written(
            request
                .with_page(page)
                .with_per_page(per_page)
                .write_query(&mut output),
            &output,
            false,
        );
    }

    let local = CertificateActionListForCertificateRequest::new(certificate_id)
        .with_statuses(statuses)
        .and_then(|request| request.with_sorts(sorts));
    if let Ok(request) = local {
        let mut output = vec![0xa5; capacity];
        check_written(
            request
                .with_page(page)
                .with_per_page(per_page)
                .write_query(&mut output),
            &output,
            false,
        );
    }
}

fn check_path(endpoint: ActionEndpoint, capacity: usize) {
    let mut output = vec![0xa5; capacity];
    check_written(endpoint.write_path(&mut output), &output, true);
}

fn check_certificate_path(endpoint: CertificateActionEndpoint, capacity: usize) {
    let mut output = vec![0xa5; capacity];
    check_written(endpoint.write_path(&mut output), &output, true);
}

fn check_written<E>(result: Result<usize, E>, output: &[u8], path: bool) {
    let Ok(len) = result else {
        return;
    };
    assert!(len <= output.len());
    let Some(written) = output.get(..len) else {
        return;
    };
    assert!(written.iter().all(u8::is_ascii));
    assert!(core::str::from_utf8(written).is_ok());
    if path {
        assert!(written.starts_with(b"/"));
    }
}

fn output_capacity(data: &[u8]) -> usize {
    let bytes = [
        data.first().copied().unwrap_or(0),
        data.get(1).copied().unwrap_or(0),
    ];
    usize::from(u16::from_le_bytes(bytes)).min(MAX_OUTPUT_BYTES)
}

fn action_ids(data: &[u8]) -> Vec<ActionId> {
    let count = usize::from(data.get(19).copied().unwrap_or(0)) % 130;
    let first = read_u64(data.get(20..).unwrap_or_default());
    (0..count)
        .filter_map(|index| {
            let value = first.wrapping_add(index as u64) % MAX_ACTION_ID;
            ActionId::new(value + 1)
        })
        .collect()
}

fn action_statuses(data: &[u8]) -> Vec<ActionStatus> {
    let count = usize::from(data.get(3).copied().unwrap_or(0)) % 5;
    (0..count)
        .map(
            |index| match data.get(4 + index).copied().unwrap_or(0) % 3 {
                0 => ActionStatus::Running,
                1 => ActionStatus::Success,
                _ => ActionStatus::Error,
            },
        )
        .collect()
}

fn action_sorts(data: &[u8]) -> Vec<(CertificateActionSortField, SortDirection)> {
    let count = usize::from(data.get(7).copied().unwrap_or(0)) % 7;
    (0..count)
        .map(|index| {
            let offset = 8 + (index * 2);
            let value = data.get(offset).copied().unwrap_or(0);
            let field = match value % 5 {
                0 => CertificateActionSortField::Id,
                1 => CertificateActionSortField::Command,
                2 => CertificateActionSortField::Status,
                3 => CertificateActionSortField::Started,
                _ => CertificateActionSortField::Finished,
            };
            let direction = if data.get(offset + 1).copied().unwrap_or(0) & 1 == 0 {
                SortDirection::Asc
            } else {
                SortDirection::Desc
            };
            (field, direction)
        })
        .collect()
}

fn read_u64(data: &[u8]) -> u64 {
    let mut bytes = [0_u8; 8];
    for (target, source) in bytes.iter_mut().zip(data.iter().copied()) {
        *target = source;
    }
    u64::from_le_bytes(bytes)
}
