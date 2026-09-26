use super::*;
use crate::{
    discovery::tests::Fixture as _,
    endpoint::OfficialCratesIoEndpoint,
    identifiers::{CrateName, NumericId, TeamLogin, UserLogin},
    query::{Include, IncludeSet, Parameter},
    wire::JsonResponsePolicy,
};
use alloc::{format, vec};
use cloud_sdk::{
    rate_limit::WallClockTimestamp,
    transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode},
};
use serde_json::{Value, json};

pub(super) const FIXTURES: [&str; 6] = [
    include_str!("fixtures/find_user.json"),
    include_str!("fixtures/get_user_stats.json"),
    include_str!("fixtures/find_team.json"),
    include_str!("fixtures/list_owners.json"),
    include_str!("fixtures/get_user_owners.json"),
    include_str!("fixtures/get_team_owners.json"),
];
pub(super) fn requests() -> [AccountRequest<'static>; 6] {
    let name = CrateName::new("serde").fixture("crate");
    [
        AccountRequest::user(UserLogin::new("ghost").fixture("login"), &[]).fixture("user"),
        AccountRequest::user_stats(NumericId::new(42).fixture("id")),
        AccountRequest::team(TeamLogin::new("github:rust-lang:crates-io").fixture("team")),
        AccountRequest::owners(name),
        AccountRequest::user_owners(name),
        AccountRequest::team_owners(name),
    ]
}
fn fixture(index: usize) -> Value {
    serde_json::from_str(FIXTURES.get(index).fixture("fixture")).fixture("json")
}
fn request(index: usize) -> AccountRequest<'static> {
    *requests().get(index).fixture("request")
}
fn decode(request: AccountRequest<'_>, wire: &[u8]) -> Result<AccountResponse, AccountError> {
    let mut bytes = vec![0xa5; wire.len()];
    let mut headers = [0xa5; 512];
    let mut response = ResponseBuffer::new(&mut bytes, wire.len(), &mut headers);
    let mut attempt = response.writer().begin_attempt().fixture("attempt");
    attempt.body_mut().fixture("body").copy_from_slice(wire);
    attempt
        .headers_mut()
        .fixture("headers")
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .fixture("media");
    attempt
        .commit(StatusCode::OK, wire.len(), ResponseMetadata::EMPTY)
        .fixture("commit");
    drop(attempt);
    let result = match JsonResponsePolicy::new(StatusCode::OK, wire.len())
        .fixture("policy")
        .admit(response, WallClockTimestamp::new(0))
    {
        Ok(success) => request.decode(OfficialCratesIoEndpoint::production_api(), success),
        Err(_) => Err(AccountError::Schema),
    };
    assert!(bytes.iter().all(|b| *b == 0));
    assert!(headers.iter().all(|b| *b == 0));
    result
}
fn changed(request: AccountRequest<'_>, value: &Value) -> Result<AccountResponse, AccountError> {
    decode(request, &serde_json::to_vec(value).fixture("json"))
}
fn put(root: &mut Value, path: &str, value: Value) {
    *root.pointer_mut(path).fixture("path") = value;
}

#[test]
fn all_source_fixtures_and_atomic_routes() {
    let expected = [
        "/api/v1/users/ghost",
        "/api/v1/users/42/stats",
        "/api/v1/teams/github%3Arust-lang%3Acrates-io",
        "/api/v1/crates/serde/owners",
        "/api/v1/crates/serde/owner_user",
        "/api/v1/crates/serde/owner_team",
    ];
    for ((request, fixture), expected) in requests().into_iter().zip(FIXTURES).zip(expected) {
        let value = decode(request, fixture.as_bytes()).fixture("decode");
        assert!(!format!("{value:?}").contains("ghost"));
        assert!(!format!("{request:?}").contains("ghost"));
        request.operation().metadata().fixture("metadata");
        request.operation().operation_id().fixture("operation");
        let mut output = [0xa5; 128];
        assert_eq!(
            request.write_target(&mut output).fixture("target").as_str(),
            expected
        );
        for size in 0..expected.len() {
            let mut short = vec![0xa5; size];
            assert!(request.write_target(&mut short).is_err());
            assert!(short.iter().all(|b| *b == 0xa5));
        }
    }
}
#[test]
fn lookups_bind_identity_and_admit_private_nulls() {
    let mut value = fixture(0);
    put(&mut value, "/user/login", json!("Ghost"));
    for path in ["/user/name", "/user/avatar", "/user/created_at"] {
        put(&mut value, path, Value::Null);
    }
    changed(request(0), &value).fixture("case and nulls");
    let canonical = AccountRequest::user(UserLogin::new("Example-User").fixture("login"), &[])
        .fixture("request");
    put(&mut value, "/user/login", json!("example_user"));
    changed(canonical, &value).fixture("canonical user");
    for login in [
        "someone-else",
        "github:rust-lang:crates-io",
        "ghost/extra",
        "",
    ] {
        put(&mut value, "/user/login", json!(login));
        assert!(changed(request(0), &value).is_err());
    }
    let mut team = fixture(2);
    for login in [
        "ghost",
        "github:rust-lang:other",
        "github:RUST-lang:crates-io",
    ] {
        put(&mut team, "/team/login", json!(login));
        assert!(changed(request(2), &team).is_err());
    }
    let mut unknown = fixture(0);
    unknown
        .as_object_mut()
        .fixture("object")
        .insert("private_extra".into(), json!({"nested":"metadata"}));
    changed(request(0), &unknown).fixture("unknown metadata");
    assert!(decode(request(0), br#"{"user":null}"#).is_err());
    assert!(decode(request(0), br#"{"user":{},"user":{}}"#).is_err());
}
#[test]
fn owner_namespaces_filter_and_duplicates() {
    let user = fixture(4).pointer("/users/0").fixture("user").clone();
    let team = fixture(5).pointer("/teams/0").fixture("team").clone();
    // Both fixture IDs are 42, but user and team IDs are disjoint namespaces.
    let value = json!({"users": [user.clone(), team.clone()]});
    let AccountResponse::Owners(list) = changed(request(3), &value).fixture("owners") else {
        unreachable!("owners response")
    };
    assert_eq!(list.items().len(), 2);
    assert_eq!(
        list.items().first().fixture("user").kind(),
        AccountKind::User
    );
    assert_eq!(
        list.items().get(1).fixture("team").kind(),
        AccountKind::Team
    );
    assert!(changed(request(4), &value).is_err());
    assert!(changed(request(5), &json!({"teams":[user.clone()]})).is_err());
    assert!(changed(request(3), &json!({"users":[team.clone(),team]})).is_err());
    let mut other = user.clone();
    put(&mut other, "/id", json!(43));
    put(&mut other, "/login", json!("GHOST"));
    assert!(changed(request(3), &json!({"users":[user.clone(),other]})).is_err());
    let mut separator_a = user.clone();
    let mut separator_b = user.clone();
    put(&mut separator_a, "/login", json!("some-user"));
    put(&mut separator_b, "/login", json!("Some_User"));
    put(&mut separator_b, "/id", json!(43));
    assert!(changed(request(3), &json!({"users":[separator_a,separator_b]})).is_err());
    let mut same_id = user.clone();
    put(&mut same_id, "/login", json!("different"));
    assert!(changed(request(3), &json!({"users":[user.clone(),same_id]})).is_err());
    for invalid in [0_i64, -1, 2_147_483_648] {
        let mut value = json!({"users":[user.clone()]});
        put(&mut value, "/users/0/id", json!(invalid));
        assert!(changed(request(3), &value).is_err());
    }
    changed(request(3), &json!({"users":[]})).fixture("empty");
}
#[test]
fn owner_count_and_metadata_bounds_do_not_truncate() {
    let user = fixture(4).pointer("/users/0").fixture("user").clone();
    let mut users = vec![];
    for n in 1..=MAX_OWNERS {
        let mut item = user.clone();
        put(&mut item, "/id", json!(n));
        put(&mut item, "/login", json!(format!("user{n}")));
        users.push(item);
    }
    let value = json!({"users":users});
    let AccountResponse::Owners(list) = changed(request(3), &value).fixture("max owners") else {
        unreachable!("owners")
    };
    assert_eq!(list.items().len(), MAX_OWNERS);
    let mut value = value;
    value
        .pointer_mut("/users")
        .fixture("users")
        .as_array_mut()
        .fixture("array")
        .push(user);
    assert!(matches!(
        changed(request(3), &value),
        Err(AccountError::Limit)
    ));
    for (path, maximum) in [
        ("/user/name", 256),
        ("/user/url", 4096),
        ("/user/avatar", 4096),
    ] {
        let mut value = fixture(0);
        put(&mut value, path, json!("x".repeat(maximum)));
        changed(request(0), &value).fixture("max metadata");
        put(
            &mut value,
            path,
            json!("x".repeat(maximum.checked_add(1).fixture("size"))),
        );
        assert!(changed(request(0), &value).is_err());
        put(&mut value, path, json!("line\nbreak"));
        assert!(changed(request(0), &value).is_err());
    }
}
#[test]
fn linked_accounts_require_explicit_include_and_unique_public_identities() {
    let include = [Include::LinkedAccounts];
    let parameters = [Parameter::Include(
        IncludeSet::new(&include).fixture("include"),
    )];
    let requested = AccountRequest::user(UserLogin::new("ghost").fixture("user"), &parameters)
        .fixture("request");
    let mut output = [0; 128];
    assert_eq!(
        requested
            .write_target(&mut output)
            .fixture("target")
            .as_str(),
        "/api/v1/users/ghost?include=linked_accounts"
    );
    let mut value = fixture(0);
    assert!(changed(requested, &value).is_err());
    let object = value.as_object_mut().fixture("object");
    object.insert("linked_accounts".into(), json!([]));
    let AccountResponse::User(user) = changed(requested, &value).fixture("empty links") else {
        unreachable!("user")
    };
    assert_eq!(
        user.linked_accounts()
            .fixture("links")
            .fixture("present")
            .len(),
        0
    );
    assert!(changed(request(0), &value).is_err());
    let item = json!({"account_id":"100", "login":"ghost", "avatar":null,"provider":"github"});
    put(&mut value, "/linked_accounts", json!([item.clone()]));
    changed(requested, &value).fixture("linked");
    for bad in [
        json!([item.clone(), item.clone()]),
        Value::Null,
        json!([{ "account_id":"100", "login":"ghost", "avatar":null, "provider":"other"}]),
    ] {
        put(&mut value, "/linked_accounts", bad);
        assert!(changed(requested, &value).is_err());
    }
    let mut links = vec![];
    for n in 0..MAX_LINKED_ACCOUNTS {
        let mut link = item.clone();
        put(&mut link, "/account_id", json!(format!("{n}")));
        put(&mut link, "/login", json!(format!("user{n}")));
        links.push(link);
    }
    put(&mut value, "/linked_accounts", json!(links));
    changed(requested, &value).fixture("max links");
    value
        .pointer_mut("/linked_accounts")
        .fixture("links")
        .as_array_mut()
        .fixture("array")
        .push(item);
    assert!(matches!(
        changed(requested, &value),
        Err(AccountError::Limit)
    ));
}
#[test]
fn statistics_validate_integer_domain_without_claiming_user_existence() {
    for n in [0, i64::MAX] {
        let AccountResponse::UserStats { total_downloads } =
            changed(request(1), &json!({"total_downloads":n})).fixture("count")
        else {
            unreachable!("count response")
        };
        assert_eq!(total_downloads, u64::try_from(n).fixture("count"));
    }
    for n in [json!(-1), json!(1.5), json!("2"), json!(u64::MAX)] {
        assert!(changed(request(1), &json!({"total_downloads":n})).is_err());
    }
}

#[test]
fn account_schemas_reject_invalid_types_missing_fields_and_timestamps() {
    for (path, invalid) in [
        ("/user/id", json!(0)),
        ("/user/id", json!("42")),
        ("/user/github_username_matches", Value::Null),
        ("/user/name", json!({})),
        ("/user/url", Value::Null),
        ("/user/created_at", json!("2026-02-30T12:00:00Z")),
    ] {
        let mut value = fixture(0);
        put(&mut value, path, invalid);
        assert!(changed(request(0), &value).is_err());
    }
    for key in [
        "id",
        "login",
        "name",
        "avatar",
        "url",
        "created_at",
        "github_username_matches",
    ] {
        let mut value = fixture(0);
        value
            .pointer_mut("/user")
            .fixture("user")
            .as_object_mut()
            .fixture("object")
            .remove(key);
        assert!(changed(request(0), &value).is_err());
    }
    for kind in [json!("unknown"), Value::Null, json!(1)] {
        let mut value = fixture(3);
        put(&mut value, "/users/0/kind", kind);
        assert!(changed(request(3), &value).is_err());
    }
}
#[cfg(all(feature = "blocking", feature = "async"))]
mod unified;
