mod dispatch_tests;
mod evidence_tests;
mod fingerprint_tests;
mod fixture;
mod separation_tests;
mod state_tests;
mod unpolled_cleanup_tests;

#[test]
fn security_tests_cannot_return_early_when_fixture_setup_fails() {
    for source in [
        include_str!("tests/dispatch_tests.rs"),
        include_str!("tests/evidence_tests.rs"),
        include_str!("tests/fingerprint_tests.rs"),
        include_str!("tests/separation_tests.rs"),
        include_str!("tests/state_tests.rs"),
        include_str!("tests/unpolled_cleanup_tests.rs"),
    ] {
        assert!(!source.contains("return;"));
        assert!(!source.contains("return }"));
    }
}
