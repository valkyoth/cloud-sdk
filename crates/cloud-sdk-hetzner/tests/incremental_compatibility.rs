//! External-consumer compatibility witnesses for published Hetzner exports.
#![cfg(feature = "serde")]

use cloud_sdk_hetzner::serde::{
    IncrementalJsonDecoder, IncrementalJsonError, IncrementalJsonEvent, IncrementalJsonLimits,
    IncrementalJsonLimitsError, IncrementalJsonProgress, IncrementalJsonVisitor, VisitControl,
};

struct Visitor(usize);

impl IncrementalJsonVisitor for Visitor {
    type Error = core::convert::Infallible;

    fn visit(&mut self, _: IncrementalJsonEvent<'_>) -> Result<VisitControl, Self::Error> {
        self.0 = self.0.saturating_add(1);
        Ok(VisitControl::Continue)
    }
}

#[test]
fn published_incremental_json_imports_and_external_visitor_remain_usable() {
    let limits: IncrementalJsonLimits = IncrementalJsonLimits::DEFAULT;
    let _: Option<IncrementalJsonLimitsError> = None;
    let mut decoder = IncrementalJsonDecoder::with_limits(limits);
    let mut visitor = Visitor(0);
    let result: Result<IncrementalJsonProgress, IncrementalJsonError<core::convert::Infallible>> =
        decoder.push(br#"{"key":true}"#, &mut visitor);
    assert!(result.is_ok());
    assert_eq!(
        decoder.finish(&mut visitor),
        Ok(IncrementalJsonProgress::Complete)
    );
    assert_eq!(visitor.0, 4);
}

#[test]
fn published_cloud_resource_kind_discriminants_are_unchanged() {
    use cloud_sdk_hetzner::serde::CloudResourceKind;
    let published = [
        CloudResourceKind::Firewall,
        CloudResourceKind::FloatingIp,
        CloudResourceKind::Image,
        CloudResourceKind::Iso,
        CloudResourceKind::LoadBalancer,
        CloudResourceKind::LoadBalancerType,
        CloudResourceKind::Network,
        CloudResourceKind::PlacementGroup,
        CloudResourceKind::PrimaryIp,
        CloudResourceKind::Server,
        CloudResourceKind::ServerType,
        CloudResourceKind::Volume,
    ];
    for (expected, actual) in published.into_iter().enumerate() {
        assert_eq!(actual as usize, expected);
    }
}
