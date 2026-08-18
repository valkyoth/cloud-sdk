use cloud_sdk::operation::{PreparationStorage, PrepareOperation};

use crate::cloud::images::ImageEndpoint;
use crate::query::{
    SourceLockedQuery, SourceQueryArgument, SourceQueryOperation, SourceQueryParameter,
    SourceQueryText,
};

use super::super::{HetznerPreparationError, HetznerPreparedOperation};

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
