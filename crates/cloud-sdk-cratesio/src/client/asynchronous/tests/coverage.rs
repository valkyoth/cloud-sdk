#[cfg(feature = "std")]
use crate::std as test_std;
use cloud_sdk::Method;

#[path = "coverage_routes.rs"]
mod routes;

fn resolve(method: Method, target: &str) -> Option<&'static str> {
    let path = target.split_once('?').map_or(target, |(path, _)| path);
    let mut selected = None;
    let mut score = 0;
    for &(identity, verb, template) in routes::ROUTES {
        if verb != method.as_str() {
            continue;
        }
        let mut concrete = path.split('/');
        let mut literal = 0usize;
        let matches = template.split('/').all(|segment| {
            let Some(value) = concrete.next() else {
                return false;
            };
            if segment.starts_with('{') && segment.ends_with('}') {
                !value.is_empty()
            } else {
                literal = literal.saturating_add(1);
                value == segment
            }
        }) && concrete.next().is_none();
        if matches && literal > score {
            selected = Some(identity);
            score = literal;
        } else if matches && literal == score {
            unreachable!("ambiguous inventory route");
        }
    }
    selected
}

pub(super) fn record(method: Method, target: &str) {
    let identity =
        resolve(method, target).unwrap_or_else(|| unreachable!("unclassified execution"));
    // Emit only the generated public identity, never paths, credentials or bodies.
    #[cfg(feature = "std")]
    {
        for mode in ["blocking", "local", "send"] {
            test_std::println!("CLOUD_SDK_EXECUTION:{identity}:{mode}");
        }
    }
    #[cfg(not(feature = "std"))]
    let _ = identity;
}

#[test]
fn inventory_resolution_prefers_literal_routes_and_rejects_unknown_execution() {
    assert_eq!(
        resolve(Method::Get, "/api/v1/crates/new"),
        Some("find_new_crate")
    );
    assert_eq!(
        resolve(Method::Get, "/api/v1/crates/serde?include=versions"),
        Some("find_crate")
    );
    assert_eq!(resolve(Method::Put, "/api/v1/crates/new"), Some("publish"));
    assert_eq!(
        resolve(Method::Get, "/api/v1/crates/serde/1.0.0/extra"),
        None
    );
    assert_eq!(resolve(Method::Delete, "/api/v1/crates/new"), None);
}
