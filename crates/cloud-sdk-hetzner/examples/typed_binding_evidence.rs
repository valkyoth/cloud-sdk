//! Emits compiled operation bindings for the release-integrity gate.

use cloud_sdk_hetzner::association::{ALL_OPERATION_EVIDENCE, ResponseIdentityClass};
use cloud_sdk_hetzner::request::ApiBaseUrl;

fn main() {
    for evidence in ALL_OPERATION_EVIDENCE {
        let descriptor = evidence.descriptor;
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            descriptor.operation_id().as_str(),
            api(descriptor.api_base_url()),
            evidence.service,
            descriptor.method().as_str(),
            descriptor.path_template(),
            evidence.endpoint_policy,
            evidence.authentication,
            evidence.authentication_scope,
            evidence.query_policy,
            evidence.body_policy,
            evidence.request_headers,
            evidence.request_media,
            evidence.success_status,
            evidence.success_shape,
            descriptor.success_root(),
            descriptor.success_required(),
            evidence.success_body,
            evidence.success_media,
            descriptor.success_body_bytes(),
            evidence.error_body,
            evidence.error_media,
            descriptor.error_body_bytes(),
            evidence.pagination,
            evidence.quota,
            evidence.retry,
            evidence.streaming,
            evidence.permit_class,
            identity(descriptor.response_identity()),
        );
    }
}

fn api(value: ApiBaseUrl) -> &'static str {
    match value {
        ApiBaseUrl::CloudV1 => "cloud",
        ApiBaseUrl::HetznerV1 => "hetzner",
    }
}

fn identity(value: ResponseIdentityClass) -> &'static str {
    match value {
        ResponseIdentityClass::None => "none",
        ResponseIdentityClass::ExactResource => "exact-resource",
        ResponseIdentityClass::ParentResource => "parent-resource",
    }
}
