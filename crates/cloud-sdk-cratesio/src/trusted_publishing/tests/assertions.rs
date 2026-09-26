use super::*;
fn check(
    policy: ExchangePolicy<'_>,
    header: &Value,
    claims: &Value,
    now: u64,
) -> Result<(), TrustedPublishingError> {
    policy.preflight(
        &serde_json::to_vec(&json!({"jwt":jwt(header, claims)})).fixture("body"),
        now,
    )
}
#[test]
fn unverified_preflight_accepts_both_sources_but_never_authenticates() {
    for p in [Publisher::GitHub, Publisher::GitLab] {
        let policy = policy(p, 100);
        // Deliberately not signed: authenticity is solely the registry's job.
        check(policy, &header(), &claims(p, 100), 100).fixture("preflight");
        assert_eq!(format!("{policy:?}"), "ExchangePolicy([redacted])");
        assert!(check(policy, &header(), &claims(p, 100), 99).is_err());
        assert!(check(policy, &header(), &claims(p, 100), 1900).is_err());
        for lifetime in [0, 1801, u64::MAX] {
            assert!(ExchangePolicy::new(config(p), "crates.io", 100, lifetime).is_err());
        }
        assert!(ExchangePolicy::new(config(p), "crates.io", u64::MAX, 1).is_err());
    }
}
#[test]
fn issuer_audience_time_required_claims_provider_and_workflow_confusion_fail() {
    for p in [Publisher::GitHub, Publisher::GitLab] {
        let policy = policy(p, 100);
        let original = claims(p, 100);
        for field in [
            "iss",
            "aud",
            "iat",
            "exp",
            "jti",
            "sha",
            if p == Publisher::GitHub {
                "run_id"
            } else {
                "job_id"
            },
            if p == Publisher::GitHub {
                "repository_owner_id"
            } else {
                "namespace_id"
            },
        ] {
            let mut c = original.clone();
            c.as_object_mut().fixture("object").remove(field);
            assert!(check(policy, &header(), &c, 100).is_err());
        }
        for (field, value) in [
            ("iss", json!("https://evil.invalid")),
            ("aud", json!("other")),
            ("aud", json!(["crates.io"])),
            ("iat", json!(101)),
            ("exp", json!(100)),
            ("exp", json!("200")),
            ("nbf", json!(101)),
            ("jti", json!("")),
        ] {
            let mut c = original.clone();
            c.as_object_mut()
                .fixture("object")
                .insert(field.into(), value);
            assert!(check(policy, &header(), &c, 100).is_err());
        }
        let (repo, workflow, numeric) = if p == Publisher::GitHub {
            ("repository", "workflow_ref", "repository_owner_id")
        } else {
            ("project_path", "ci_config_ref_uri", "namespace_id")
        };
        for (field, value) in [
            (repo, "other/regex"),
            (repo, "rust-lang/other"),
            (
                workflow,
                "rust-lang/regex/.github/workflows/other.yml@refs/heads/main",
            ),
            (numeric, "0"),
            (numeric, "-1"),
            (numeric, "18446744073709551616"),
        ] {
            let mut c = original.clone();
            c.as_object_mut()
                .fixture("object")
                .insert(field.into(), json!(value));
            assert!(check(policy, &header(), &c, 100).is_err());
        }
        for field in ["alg", "kid"] {
            let mut h = header();
            h.as_object_mut().fixture("object").remove(field);
            assert!(check(policy, &h, &original, 100).is_err());
        }
        for (field, value) in [
            ("alg", json!("none")),
            ("alg", json!("HS256")),
            ("kid", json!("")),
            ("crit", json!([])),
            ("b64", json!(true)),
        ] {
            let mut h = header();
            h.as_object_mut()
                .fixture("object")
                .insert(field.into(), value);
            assert!(check(policy, &h, &original, 100).is_err());
        }
        let other = if p == Publisher::GitHub {
            Publisher::GitLab
        } else {
            Publisher::GitHub
        };
        assert!(check(policy, &header(), &claims(other, 100), 100).is_err());
        let c = config(p);
        let c = PublisherConfig::new(
            p,
            c.krate,
            c.owner,
            c.project,
            c.workflow,
            Some("production"),
        )
        .fixture("environment");
        let bound = ExchangePolicy::new(c, "crates.io", 100, 600).fixture("policy");
        assert!(check(bound, &header(), &original, 100).is_err());
        let mut env = original.clone();
        env.as_object_mut()
            .fixture("object")
            .insert("environment".into(), json!("staging"));
        assert!(check(bound, &header(), &env, 100).is_err());
        env.as_object_mut()
            .fixture("object")
            .insert("environment".into(), json!("PRODUCTION"));
        check(bound, &header(), &env, 100).fixture("case fold");
    }
    for event in ["pull_request_target", "workflow_run"] {
        let mut c = claims(Publisher::GitHub, 100);
        c.as_object_mut()
            .fixture("object")
            .insert("event_name".into(), json!(event));
        assert!(check(policy(Publisher::GitHub, 100), &header(), &c, 100).is_err());
    }
}
#[test]
fn malformed_or_oversized_compact_jwt_and_duplicate_claims_fail() {
    let p = policy(Publisher::GitHub, 100);
    for text in [
        String::from("a.b.c"),
        String::from("..."),
        "A".repeat(16385),
        format!(
            "{}.{}.A",
            encode(&serde_json::to_vec(&header()).fixture("header")),
            encode(&serde_json::to_vec(&claims(Publisher::GitHub, 100)).fixture("claims"))
        ),
    ] {
        assert!(
            p.preflight(
                &serde_json::to_vec(&json!({"jwt":text})).fixture("body"),
                100
            )
            .is_err()
        );
    }
    let duplicate = encode(br#"{"iss":"bad","iss":"other"}"#);
    assert!(p.preflight(&serde_json::to_vec(&json!({"jwt":format!("{}.{}.{}", encode(&serde_json::to_vec(&header()).fixture("header")), duplicate, encode(b"fake"))})).fixture("body"),100).is_err());
    let mut oversized = vec![b'a'; 16385];
    assert!(OidcAssertion::from_mut_bytes(CredentialOrigin::Production, &mut oversized).is_err());
    assert!(oversized.iter().all(|b| *b == 0));
}
