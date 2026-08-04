#[test]
fn security_sensitive_tests_cannot_return_early_after_fixture_failures() {
    for source in [
        include_str!("dynamic.rs"),
        include_str!("local_async.rs"),
        include_str!("prepared.rs"),
        include_str!("raw_fault.rs"),
        include_str!("script.rs"),
        include_str!("stream.rs"),
    ] {
        assert!(!source.contains("return;"));
        assert!(!source.contains("return }"));
    }
}
