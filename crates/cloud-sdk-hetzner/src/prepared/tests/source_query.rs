use cloud_sdk::operation::{PreparationStorage, PrepareOperation};

use crate::cloud::images::ImageEndpoint;
use crate::query::{
    SourceLockedQuery, SourceQueryArgument, SourceQueryOperation, SourceQueryParameter,
    SourceQueryText,
};

use super::super::{HetznerPreparationError, HetznerPreparedOperation};

#[test]
fn network_members_prepare_repeated_filters_and_reject_wrong_operation() {
    use crate::cloud::networks::{NetworkId, NetworkMembersEndpoint};
    let id = NetworkId::new(42).unwrap_or_else(|| unreachable!("id"));
    let endpoint = NetworkMembersEndpoint::new(id);
    let text = |value| SourceQueryText::new(value).unwrap_or_else(|_| unreachable!("text"));
    let arguments = [
        SourceQueryArgument::text(SourceQueryParameter::Type, text("server")),
        SourceQueryArgument::text(SourceQueryParameter::Type, text("load_balancer")),
        SourceQueryArgument::subnet(text("10.0.1.0/24")),
        SourceQueryArgument::subnet(text("10.0.2.0/24")),
        SourceQueryArgument::text(SourceQueryParameter::Status, text("ok")),
        SourceQueryArgument::text(SourceQueryParameter::Status, text("error")),
        SourceQueryArgument::text(SourceQueryParameter::Sort, text("id:asc")),
        SourceQueryArgument::text(SourceQueryParameter::Sort, text("type:desc")),
        SourceQueryArgument::integer(SourceQueryParameter::Page, 2),
        SourceQueryArgument::integer(SourceQueryParameter::PerPage, 10),
    ];
    let query = SourceLockedQuery::try_new(SourceQueryOperation::LIST_NETWORK_MEMBERS, &arguments)
        .unwrap_or_else(|_| unreachable!("query"));
    let operation = HetznerPreparedOperation::query(endpoint, query);
    let mut target = [0; 512];
    let mut body = [0; 1];
    let prepared = operation
        .prepare(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("preparation"));
    assert_eq!(
        prepared.transport_request().target().as_str(),
        "/networks/42/members?page=2&per_page=10&sort=id%3Aasc&sort=type%3Adesc&status=ok&status=error&subnet=10.0.1.0%2F24&subnet=10.0.2.0%2F24&type=server&type=load_balancer"
    );
    let mut short = [0xa5; 8];
    assert!(
        operation
            .prepare(PreparationStorage::new(&mut short, &mut body))
            .is_err()
    );
    assert!(short.iter().all(|byte| *byte == 0));
    let wrong = HetznerPreparedOperation::query(ImageEndpoint::List, query);
    assert!(matches!(
        wrong.prepare(PreparationStorage::new(&mut target, &mut body)),
        Err(HetznerPreparationError::OperationMismatch)
    ));
    let invalid = [SourceQueryArgument::text(
        SourceQueryParameter::Type,
        text("future-kind"),
    )];
    assert!(
        SourceLockedQuery::try_new(SourceQueryOperation::LIST_NETWORK_MEMBERS, &invalid).is_err()
    );
}

#[test]
fn source_locked_query_prepares_complete_repeated_filters() {
    let image_type = SourceQueryText::new("system");
    let status = SourceQueryText::new("available");
    let (Ok(image_type), Ok(status)) = (image_type, status) else {
        unreachable!("security fixture construction failed");
    };
    let arguments = [
        SourceQueryArgument::text(SourceQueryParameter::Status, status),
        SourceQueryArgument::text(SourceQueryParameter::Type, image_type),
    ];
    let query = SourceLockedQuery::try_new(SourceQueryOperation::LIST_IMAGES, &arguments);
    let Ok(query) = query else {
        unreachable!("security fixture construction failed");
    };
    let operation = HetznerPreparedOperation::query(ImageEndpoint::List, query);
    let mut target = [0_u8; 96];
    let mut body = [0_u8; 1];
    let prepared = operation.prepare(PreparationStorage::new(&mut target, &mut body));
    let Ok(prepared) = prepared else {
        unreachable!("security fixture preparation failed");
    };
    assert_eq!(
        prepared.transport_request().target().as_str(),
        "/images?status=available&type=system"
    );

    target.fill(0xa5);
    body.fill(0x5a);
    let empty = [];
    let server_query = SourceLockedQuery::try_new(SourceQueryOperation::LIST_SERVERS, &empty)
        .unwrap_or_else(|_| unreachable!());
    let mismatch = HetznerPreparedOperation::query(ImageEndpoint::List, server_query);
    assert!(matches!(
        mismatch.prepare(PreparationStorage::new(&mut target, &mut body)),
        Err(HetznerPreparationError::OperationMismatch)
    ));
    assert!(target.iter().all(|byte| *byte == 0));
    assert!(body.iter().all(|byte| *byte == 0));
}
