use super::*;
use crate::cloud::networks::primary_ips::PrimaryIpId;
use crate::prepared::{BodyWire, EndpointWire, HetznerPreparationError, ResponseProfile};

fn request() -> LoadBalancerCreateRequest<'static> {
    let (Ok(name), Ok(kind)) = (LoadBalancerName::new("edge"), LoadBalancerType::new("lb11"))
    else {
        unreachable!("load balancer fixture must be valid");
    };
    LoadBalancerCreateRequest::new(name, kind)
}

#[test]
fn existing_primary_ips_and_system_managed_intent_are_encoded() {
    let Some(id) = PrimaryIpId::new(42) else {
        unreachable!("primary IP fixture must be valid");
    };
    for (ipv4, ipv6, expected) in [
        (Some(id), None, br#"{"public_net":{"ipv4":42,"ipv6":null},"name":"edge","load_balancer_type":"lb11"}"#.as_slice()),
        (None, Some(id), br#"{"public_net":{"ipv4":null,"ipv6":42},"name":"edge","load_balancer_type":"lb11"}"#.as_slice()),
        (None, None, br#"{"public_net":{"ipv4":null,"ipv6":null},"name":"edge","load_balancer_type":"lb11"}"#.as_slice()),
    ] {
        let mut output = [0_u8; 256];
        let result = request().with_public_net(ipv4, ipv6).write_body(&mut output);
        let Ok(len) = result else {
            unreachable!("valid Primary IP selection must encode");
        };
        assert_eq!(output.get(..len), Some(expected));
    }
    let mut output = [0_u8; 128];
    let Ok(len) = request().write_body(&mut output) else {
        unreachable!("default request must encode");
    };
    assert_eq!(
        output.get(..len),
        Some(br#"{"name":"edge","load_balancer_type":"lb11"}"#.as_slice())
    );
}

#[test]
fn disabled_public_interface_rejects_primary_ip_options_in_either_order() {
    let first = request()
        .with_public_net(None, None)
        .with_public_interface(false);
    let last = request()
        .with_public_interface(false)
        .with_public_net(None, None);
    for request in [first, last] {
        let mut output = [0_u8; 256];
        assert_eq!(
            request.write_body(&mut output),
            Err(HetznerPreparationError::Body)
        );
    }
    let mut short = [0_u8; 10];
    assert!(
        request()
            .with_public_net(None, None)
            .write_body(&mut short)
            .is_err()
    );
}

#[test]
fn deletion_expects_created_action_not_empty_success() {
    let Some(id) = LoadBalancerId::new(42) else {
        unreachable!("load balancer fixture must be valid");
    };
    assert_eq!(
        LoadBalancerEndpoint::Delete(id).response_profile(),
        ResponseProfile::JsonCreated
    );
}
