//! Emits compiled operation descriptors for the release-integrity gate.

use cloud_sdk_hetzner::association::{
    ALL_OPERATIONS, AuthenticationClass, BodyPolicy, PaginationPolicy, PermitClass, QueryPolicy,
    ResponseIdentityClass, ResponseShape, RetryPolicy,
};
use cloud_sdk_hetzner::request::ApiBaseUrl;

fn main() {
    for descriptor in ALL_OPERATIONS {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            descriptor.operation_id().as_str(),
            descriptor.method().as_str(),
            descriptor.path_template(),
            endpoint(descriptor.api_base_url()),
            authentication(descriptor.authentication()),
            query(descriptor.query_policy()),
            body(descriptor.body_policy()),
            descriptor.success_status().get(),
            response(descriptor.response_shape()),
            descriptor.success_body_bytes(),
            descriptor.error_body_bytes(),
            pagination(descriptor.pagination()),
            retry(descriptor.retry()),
            permit(descriptor.permit()),
            identity(descriptor.response_identity()),
        );
    }
}

fn endpoint(value: ApiBaseUrl) -> &'static str {
    match value {
        ApiBaseUrl::CloudV1 => "cloud-v1",
        ApiBaseUrl::HetznerV1 => "console-v1",
    }
}

fn authentication(value: AuthenticationClass) -> &'static str {
    match value {
        AuthenticationClass::Bearer => "bearer",
        AuthenticationClass::Basic => "basic",
        _ => "unknown",
    }
}

fn query(value: QueryPolicy) -> &'static str {
    match value {
        QueryPolicy::Forbidden => "forbidden",
        QueryPolicy::Optional => "optional",
        QueryPolicy::Required => "required",
        _ => "unknown",
    }
}

fn body(value: BodyPolicy) -> &'static str {
    match value {
        BodyPolicy::Forbidden => "forbidden",
        BodyPolicy::RequiredJson => "json",
        _ => "unknown",
    }
}

fn response(value: ResponseShape) -> &'static str {
    match value {
        ResponseShape::Empty => "empty",
        ResponseShape::Action => "action",
        ResponseShape::Actions => "actions",
        ResponseShape::ActionsPage => "actions-page",
        ResponseShape::Resource => "resource",
        ResponseShape::ResourceList => "resource-list",
        ResponseShape::ResourcePage => "resource-page",
        ResponseShape::Composite => "composite",
        ResponseShape::Metrics => "metrics",
        ResponseShape::ZoneFile => "zonefile",
        ResponseShape::Pricing => "pricing",
        ResponseShape::Folders => "folders",
        _ => "unknown",
    }
}

fn pagination(value: PaginationPolicy) -> &'static str {
    match value {
        PaginationPolicy::None => "none",
        PaginationPolicy::Numbered => "numbered",
        _ => "unknown",
    }
}

fn retry(value: RetryPolicy) -> &'static str {
    match value {
        RetryPolicy::Never => "never",
        RetryPolicy::Explicit => "explicit",
        _ => "unknown",
    }
}

fn permit(value: PermitClass) -> &'static str {
    match value {
        PermitClass::None => "none",
        PermitClass::Mutation => "mutation",
        PermitClass::Destructive => "destructive",
        PermitClass::Cost => "cost",
        _ => "unknown",
    }
}

fn identity(value: ResponseIdentityClass) -> &'static str {
    match value {
        ResponseIdentityClass::None => "none",
        ResponseIdentityClass::ExactResource => "exact-resource",
        ResponseIdentityClass::ParentResource => "parent-resource",
    }
}
