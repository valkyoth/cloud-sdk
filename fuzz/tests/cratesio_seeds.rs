#[path = "../support/cratesio_continuation.rs"]
mod continuation;
#[path = "../support/cratesio_metadata.rs"]
mod metadata;
#[path = "../support/cratesio_redirect.rs"]
mod redirect;
#[path = "../support/cratesio_targets.rs"]
mod targets;

fn line(seed: &[u8]) -> &[u8] {
    seed.strip_suffix(b"\n").expect("text corpus line ending")
}

#[test]
fn redirect_provenance_binds_archive_and_clears_storage() {
    assert!(redirect::exercise(line(include_bytes!(
        "../seeds/cratesio_redirect/valid"
    ))));
    for value in [
        b"https://evil.invalid/crates/serde/serde-1.0.0.crate".as_slice(),
        b"https://static.crates.io/crates/serde/serde-2.0.0.crate",
        b"https://static.crates.io/crates/serde/serde-1.0.0.crate?token=x",
        b"https://static.crates.io@evil.invalid/crates/serde/serde-1.0.0.crate",
    ] {
        assert!(!redirect::exercise(value));
    }
}

#[test]
fn named_seeds_reach_success_and_rejection() {
    assert_eq!(
        targets::exercise(line(include_bytes!("../seeds/cratesio_targets/name"))),
        (true, true)
    );
    assert_eq!(
        targets::exercise(line(include_bytes!("../seeds/cratesio_targets/query"))),
        (false, true)
    );
    assert!(!targets::exercise(line(include_bytes!("../seeds/cratesio_targets/rejected"))).0);
    assert!(continuation::exercise(line(include_bytes!(
        "../seeds/cratesio_continuation/next"
    ))));
    assert!(!continuation::exercise(line(include_bytes!(
        "../seeds/cratesio_continuation/foreign"
    ))));
    assert!(metadata::exercise(include_bytes!(
        "../seeds/cratesio_metadata/valid"
    )));
    assert!(!metadata::exercise(include_bytes!(
        "../seeds/cratesio_metadata/duplicate"
    )));
}

#[test]
fn bounded_edges_are_deterministic() {
    assert_eq!(targets::exercise(&[0xff; 1025]), (false, false));
    assert!(!continuation::exercise(b"?page=1"));
    assert!(!continuation::exercise(b"?page=11"));
    assert!(!continuation::exercise(b"?page=2&page=3"));
    assert!(!continuation::exercise(&vec![b'a'; 4097]));
    assert!(!metadata::exercise(&vec![b' '; 131073]));
    assert!(!metadata::exercise(b"{}"));
}
