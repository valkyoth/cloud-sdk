//! Prepares a source-locked repeated Image query without a network client.

use cloud_sdk_hetzner::cloud::images::ImageEndpoint;
use cloud_sdk_hetzner::prepared::HetznerPreparedOperation;
use cloud_sdk_hetzner::query::{
    SourceLockedQuery, SourceQueryArgument, SourceQueryError, SourceQueryOperation,
    SourceQueryParameter, SourceQueryText,
};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let available = SourceQueryText::new("available")?;
    let system = SourceQueryText::new("system")?;
    let snapshot = SourceQueryText::new("snapshot")?;
    let arguments = [
        SourceQueryArgument::text(SourceQueryParameter::Status, available),
        SourceQueryArgument::text(SourceQueryParameter::Type, system),
        SourceQueryArgument::text(SourceQueryParameter::Type, snapshot),
    ];
    let query = SourceLockedQuery::try_new(SourceQueryOperation::LIST_IMAGES, &arguments)?;
    let operation = HetznerPreparedOperation::query(ImageEndpoint::List, query);

    let mut output = [0_u8; 96];
    let written = query.write_query(&mut output)?;
    let encoded = output
        .get(..written)
        .ok_or(SourceQueryError::CorruptContract)?;
    assert_eq!(
        core::str::from_utf8(encoded)?,
        "status=available&type=system&type=snapshot"
    );

    let _ = operation;
    Ok(())
}
