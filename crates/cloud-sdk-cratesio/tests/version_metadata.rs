//! External-consumer coverage for protected dynamic metadata enumeration.
use cloud_sdk::{
    rate_limit::WallClockTimestamp,
    transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode},
};
use cloud_sdk_cratesio::{
    discovery::DiscoveryError,
    endpoint::OfficialCratesIoEndpoint,
    identifiers::CrateName,
    query::{Include, IncludeSet, Parameter, PerPage},
    versions::{VersionRequest, VersionResponse},
    wire::JsonResponsePolicy,
};
use serde_json::{Value, json};

#[test]
fn external_consumer_enumerates_dynamic_metadata() -> Result<(), Box<dyn core::error::Error>> {
    let mut value: Value =
        serde_json::from_str(include_str!("../src/versions/fixtures/list_versions.json"))?;
    *value
        .pointer_mut("/versions/0/features")
        .ok_or(DiscoveryError::Schema)? = json!({"future-feature": ["dep:new"], "another": []});
    let meta = value
        .get_mut("meta")
        .and_then(Value::as_object_mut)
        .ok_or(DiscoveryError::Schema)?;
    meta.insert(
        "release_tracks".into(),
        json!({"1": {"highest": "1.0.0"}, "2": {"highest": "2.0.0"}}),
    );
    meta.insert("future_empty".into(), json!({}));
    let wire = serde_json::to_vec(&value)?;
    let mut bytes = vec![0xa5; wire.len()];
    let mut headers = [0xa5; 512];
    let mut response = ResponseBuffer::new(&mut bytes, wire.len(), &mut headers);
    let mut attempt = response.writer().begin_attempt()?;
    attempt.body_mut()?.copy_from_slice(&wire);
    attempt.headers_mut()?.try_push(
        "content-type",
        b"application/json",
        HeaderSensitivity::Public,
    )?;
    attempt.commit(StatusCode::OK, wire.len(), ResponseMetadata::EMPTY)?;
    drop(attempt);
    let tracks = [Include::ReleaseTracks];
    let parameters = [
        Parameter::PerPage(PerPage::DEFAULT),
        Parameter::Include(IncludeSet::new(&tracks)?),
    ];
    let request = VersionRequest::list(CrateName::new("serde")?, &parameters)?;
    let success = JsonResponsePolicy::new(StatusCode::OK, wire.len())?
        .admit(response, WallClockTimestamp::new(0))?;
    let VersionResponse::Versions(page) =
        request.decode(OfficialCratesIoEndpoint::production_api(), success)?
    else {
        unreachable!("version fixture returned wrong response variant");
    };
    let features = page
        .versions
        .first()
        .ok_or(DiscoveryError::Schema)?
        .fields()
        .get("features")?
        .ok_or(DiscoveryError::Schema)?;
    let mut names = Vec::new();
    features.visit_fields(|name, value| {
        names.push((name.to_owned(), value.array()?.len()));
        Ok(())
    })?;
    names.sort();
    assert_eq!(names, [("another".into(), 0), ("future-feature".into(), 1)]);
    let tracks = page
        .meta
        .get("release_tracks")?
        .ok_or(DiscoveryError::Schema)?;
    let mut releases = Vec::new();
    tracks.visit_fields(|name, value| {
        let highest = value.get("highest")?.ok_or(DiscoveryError::Schema)?;
        releases.push((name.to_owned(), highest.with_text(str::to_owned)?));
        Ok(())
    })?;
    releases.sort();
    assert_eq!(
        releases,
        [("1".into(), "1.0.0".into()), ("2".into(), "2.0.0".into())]
    );
    let mut calls = 0;
    assert_eq!(
        features.visit_fields(|_, _| {
            calls += 1;
            Err(DiscoveryError::Limit)
        }),
        Err(DiscoveryError::Limit)
    );
    assert_eq!(calls, 1);
    let total = page.meta.get("total")?.ok_or(DiscoveryError::Schema)?;
    page.meta
        .get("future_empty")?
        .ok_or(DiscoveryError::Schema)?
        .visit_fields(|_, _| unreachable!("empty object visitor ran"))?;
    assert_eq!(
        total.visit_fields(|_, _| unreachable!("non-object visitor ran")),
        Err(DiscoveryError::Schema)
    );
    assert_eq!(format!("{features:?}"), "DiscoveryValue([redacted])");
    assert_eq!(format!("{tracks:?}"), "DiscoveryValue([redacted])");
    assert!(bytes.iter().all(|byte| *byte == 0));
    assert!(headers.iter().all(|byte| *byte == 0));
    Ok(())
}
