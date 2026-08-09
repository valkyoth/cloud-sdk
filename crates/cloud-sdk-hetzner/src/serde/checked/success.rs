use super::*;

pub(super) fn decode_checked_success(
    operation: &str,
    binding: ResponseBinding,
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
    quota: HetznerQuota,
) -> Result<CheckedHetznerResponse, HetznerDecodeError> {
    if checked.status().get() != binding.status {
        return Err(HetznerDecodeError::ResponsePolicy(
            ResponsePolicyError::UnexpectedStatus,
        ));
    }
    let success = if binding.shape == ResponseShape::Empty {
        HetznerSuccess::Empty
    } else {
        let bytes = ResponseBytes::new(checked.body()).map_err(HetznerDecodeError::ResponseSize)?;
        if matches!(
            operation,
            "list_certificates"
                | "list_ssh_keys"
                | "list_storage_boxes"
                | "list_zones"
                | "list_zone_rrsets"
                | "get_zone_zonefile"
        ) || operation.contains("storage_box")
        {
            validate_incremental(bytes.as_slice())?;
        }
        let mut value =
            strict_json::parse_with_scratch(bytes.as_slice(), workspace.decoder_scratch_mut())
                .map_err(map_json_error)?;
        decode_success(operation, binding, &mut value).map_err(HetznerDecodeError::Model)?
    };
    Ok(CheckedHetznerResponse { success, quota })
}

pub(super) fn decode_provider_error(
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
    quota: HetznerQuota,
) -> Result<HetznerApiError, HetznerDecodeError> {
    if response.body().is_empty() {
        return Err(HetznerDecodeError::MissingErrorBody);
    }
    if response.body().len() > MAX_SERDE_RESPONSE_BYTES {
        return Err(HetznerDecodeError::ResponseSize(
            ResponseSizeError::TooLarge,
        ));
    }
    let content_type = response
        .content_type()
        .map_err(|_| HetznerDecodeError::ErrorContentType)?
        .ok_or(HetznerDecodeError::ErrorContentType)?;
    if !content_type.matches(MediaType::JSON) {
        return Err(HetznerDecodeError::ErrorContentType);
    }
    let mut value =
        strict_json::parse_with_scratch(response.body(), workspace.decoder_scratch_mut())
            .map_err(map_json_error)?;
    let envelope = object_mut(&mut value).map_err(HetznerDecodeError::Model)?;
    let error = object_mut(required_mut(envelope, "error").map_err(HetznerDecodeError::Model)?)
        .map_err(HetznerDecodeError::Model)?;
    let code = value_text(
        required(error, "code").map_err(HetznerDecodeError::Model)?,
        128,
    )
    .map_err(HetznerDecodeError::Model)?;
    if !valid_error_code(&code, 128) {
        return Err(HetznerDecodeError::Model(ResponseModelError::InvalidText));
    }
    let message = required_mut(error, "message")
        .map_err(HetznerDecodeError::Model)?
        .take_string()
        .map(SensitiveText::new)
        .ok_or(HetznerDecodeError::Model(ResponseModelError::WrongType))?;
    message
        .validate(16_384)
        .map_err(HetznerDecodeError::Model)?;
    Ok(HetznerApiError {
        code: ApiErrorCode::from_api_str(&code),
        code_text: code,
        message,
        quota,
    })
}

fn decode_success(
    operation: &str,
    binding: ResponseBinding,
    value: &mut Value,
) -> Result<HetznerSuccess, ResponseModelError> {
    {
        let envelope = object(value)?;
        validate_required(envelope, binding.required)?;
    }
    if binding.shape == ResponseShape::Composite {
        return decode_composite(operation, binding, object_mut(value)?);
    }
    if binding.shape == ResponseShape::ZoneFile {
        let envelope = object_mut(value)?;
        return parse_zonefile(required_mut(envelope, "zonefile")?).map(HetznerSuccess::ZoneFile);
    }
    match operation {
        "list_locations" => return parse_location_page(value).map(HetznerSuccess::Locations),
        "get_location" => {
            return parse_location(required(object(value)?, "location")?)
                .map(HetznerSuccess::Location);
        }
        "list_storage_boxes" => {
            return parse_storage_box_page(value).map(HetznerSuccess::StorageBoxes);
        }
        "get_storage_box" | "update_storage_box" => {
            return parse_storage_box(required_mut(object_mut(value)?, "storage_box")?)
                .map(HetznerSuccess::StorageBox);
        }
        "list_storage_box_types" => {
            return parse_storage_box_type_page(value).map(HetznerSuccess::StorageBoxTypes);
        }
        "get_storage_box_type" => {
            return parse_storage_box_type(required_mut(object_mut(value)?, "storage_box_type")?)
                .map(HetznerSuccess::StorageBoxType);
        }
        "list_storage_box_snapshots" => {
            return parse_storage_box_snapshots(required_mut(object_mut(value)?, "snapshots")?)
                .map(HetznerSuccess::StorageBoxSnapshots);
        }
        "get_storage_box_snapshot" | "update_storage_box_snapshot" => {
            return parse_storage_box_snapshot(required_mut(object_mut(value)?, "snapshot")?)
                .map(HetznerSuccess::StorageBoxSnapshot);
        }
        "list_storage_box_subaccounts" => {
            return parse_storage_box_subaccounts(required_mut(object_mut(value)?, "subaccounts")?)
                .map(HetznerSuccess::StorageBoxSubaccounts);
        }
        "get_storage_box_subaccount" | "update_storage_box_subaccount" => {
            return parse_storage_box_subaccount(required_mut(object_mut(value)?, "subaccount")?)
                .map(HetznerSuccess::StorageBoxSubaccount);
        }
        _ => {}
    }
    match binding.shape {
        ResponseShape::Empty => Ok(HetznerSuccess::Empty),
        ResponseShape::Action => {
            parse_action(required_mut(object_mut(value)?, "action")?).map(HetznerSuccess::Action)
        }
        ResponseShape::Actions | ResponseShape::ActionsPage => {
            let envelope = object_mut(value)?;
            let pagination = if binding.shape == ResponseShape::ActionsPage {
                Some(parse_pagination(required(envelope, "meta")?)?)
            } else {
                None
            };
            if let Some(page) = &pagination {
                validate_page_item_count(required(envelope, "actions")?, page)?;
            }
            let actions = parse_actions(required_mut(envelope, "actions")?)?;
            Ok(HetznerSuccess::Actions {
                actions,
                pagination,
            })
        }
        ResponseShape::Resource | ResponseShape::ResourceList | ResponseShape::ResourcePage => {
            decode_resources(binding, object_mut(value)?)
        }
        ResponseShape::Composite => Err(ResponseModelError::EnvelopeMismatch),
        ResponseShape::Metrics => {
            parse_metrics(required_mut(object_mut(value)?, "metrics")?).map(HetznerSuccess::Metrics)
        }
        ResponseShape::ZoneFile => Err(ResponseModelError::EnvelopeMismatch),
        ResponseShape::Pricing => {
            parse_pricing(required(object(value)?, "pricing")?).map(HetznerSuccess::Pricing)
        }
        ResponseShape::Folders => {
            parse_folders(required(object(value)?, "folders")?).map(HetznerSuccess::Folders)
        }
    }
}

fn validate_incremental(bytes: &[u8]) -> Result<(), HetznerDecodeError> {
    struct ValidationVisitor;

    impl IncrementalJsonVisitor for ValidationVisitor {
        type Error = core::convert::Infallible;

        fn visit(&mut self, _event: IncrementalJsonEvent<'_>) -> Result<VisitControl, Self::Error> {
            Ok(VisitControl::Continue)
        }
    }

    let mut decoder = IncrementalJsonDecoder::new();
    let mut visitor = ValidationVisitor;
    for chunk in bytes.chunks(257) {
        if decoder.push(chunk, &mut visitor).is_err() {
            return Err(HetznerDecodeError::MalformedPayload);
        }
    }
    match decoder.finish(&mut visitor) {
        Ok(IncrementalJsonProgress::Complete) => Ok(()),
        _ => Err(HetznerDecodeError::MalformedPayload),
    }
}

fn decode_resources(
    binding: ResponseBinding,
    envelope: &mut Map,
) -> Result<HetznerSuccess, ResponseModelError> {
    if is_dns_resource_root(binding.root) {
        let pagination = if binding.shape == ResponseShape::ResourcePage {
            Some(parse_pagination(required(envelope, "meta")?)?)
        } else {
            None
        };
        if let Some(page) = &pagination {
            validate_page_item_count(required(envelope, binding.root)?, page)?;
        }
        if binding.shape == ResponseShape::Resource {
            return parse_dns_resource(binding.root, required_mut(envelope, binding.root)?)
                .map(HetznerSuccess::DnsResource);
        }
        let resources = parse_dns_resources(binding.root, required_mut(envelope, binding.root)?)?;
        return Ok(HetznerSuccess::DnsResources {
            resources,
            pagination,
        });
    }
    if is_security_resource_root(binding.root) {
        let pagination = if binding.shape == ResponseShape::ResourcePage {
            Some(parse_pagination(required(envelope, "meta")?)?)
        } else {
            None
        };
        if let Some(page) = &pagination {
            validate_page_item_count(required(envelope, binding.root)?, page)?;
        }
        if binding.shape == ResponseShape::Resource {
            return parse_security_resource(binding.root, required_mut(envelope, binding.root)?)
                .map(HetznerSuccess::SecurityResource);
        }
        let resources =
            parse_security_resources(binding.root, required_mut(envelope, binding.root)?)?;
        return Ok(HetznerSuccess::SecurityResources {
            resources,
            pagination,
        });
    }
    let value = required(envelope, binding.root)?;
    if is_cloud_resource_root(binding.root) {
        if binding.shape == ResponseShape::Resource {
            return parse_cloud_resource(binding.root, value).map(HetznerSuccess::CloudResource);
        }
        let pagination = if binding.shape == ResponseShape::ResourcePage {
            Some(parse_pagination(required(envelope, "meta")?)?)
        } else {
            None
        };
        if let Some(page) = &pagination {
            validate_page_item_count(value, page)?;
        }
        let resources = parse_cloud_resources(binding.root, value)?;
        return Ok(HetznerSuccess::CloudResources {
            resources,
            pagination,
        });
    }
    if binding.shape == ResponseShape::Resource {
        return parse_resource(binding.root, value).map(HetznerSuccess::Resource);
    }
    let pagination = if binding.shape == ResponseShape::ResourcePage {
        Some(parse_pagination(required(envelope, "meta")?)?)
    } else {
        None
    };
    if let Some(page) = &pagination {
        validate_page_item_count(value, page)?;
    }
    let resources = parse_resources(binding.root, value)?;
    Ok(HetznerSuccess::Resources {
        resources,
        pagination,
    })
}

fn decode_composite(
    operation: &str,
    binding: ResponseBinding,
    envelope: &mut Map,
) -> Result<HetznerSuccess, ResponseModelError> {
    let (secrets, null_secrets) = take_composite_secrets(operation, envelope)?;
    let cloud_resource = if binding.root != "-" && is_cloud_resource_root(binding.root) {
        envelope
            .get(binding.root)
            .map(|value| parse_cloud_resource(binding.root, value))
            .transpose()?
    } else {
        None
    };
    let dns_resource = if binding.root != "-" && is_dns_resource_root(binding.root) {
        envelope
            .get_mut(binding.root)
            .map(|value| parse_dns_resource(binding.root, value))
            .transpose()?
    } else {
        None
    };
    let has_dns_resource = dns_resource.is_some();
    let mut dns_resources = Vec::new();
    if let Some(dns_resource) = dns_resource {
        dns_resources
            .try_reserve_exact(1)
            .map_err(|_| ResponseModelError::Allocation)?;
        dns_resources.push(dns_resource);
    }
    let security_resource = if binding.root != "-" && is_security_resource_root(binding.root) {
        envelope
            .get_mut(binding.root)
            .map(|value| parse_security_resource(binding.root, value))
            .transpose()?
    } else {
        None
    };
    let storage_box_resource = if matches!(
        operation,
        "create_storage_box" | "create_storage_box_snapshot" | "create_storage_box_subaccount"
    ) {
        envelope
            .get_mut(binding.root)
            .map(|value| parse_storage_box_composite_resource(operation, value))
            .transpose()?
    } else {
        None
    };
    let resource = if binding.root == "-"
        || cloud_resource.is_some()
        || has_dns_resource
        || security_resource.is_some()
        || storage_box_resource.is_some()
    {
        None
    } else {
        envelope
            .get(binding.root)
            .map(|value| parse_resource(binding.root, value))
            .transpose()?
    };
    let action = envelope.get_mut("action").map(parse_action).transpose()?;
    let actions = envelope
        .get_mut("actions")
        .map(parse_actions)
        .transpose()?
        .unwrap_or_default();
    let next_actions = envelope
        .get_mut("next_actions")
        .map(parse_actions)
        .transpose()?
        .unwrap_or_default();
    Ok(HetznerSuccess::Composite(CompositeResult {
        resource,
        cloud_resource,
        dns_resources,
        security_resource,
        storage_box_resource,
        action,
        actions,
        next_actions,
        secrets,
        null_secrets,
    }))
}

fn take_composite_secrets(
    operation: &str,
    envelope: &mut Map,
) -> Result<(Vec<NamedSensitiveText>, Vec<&'static str>), ResponseModelError> {
    let mut secrets = Vec::new();
    let mut null_secrets = Vec::new();
    for key in ["root_password", "password", "wss_url"] {
        if let Some(value) = envelope.get_mut(key) {
            let nullable = secret_policy(operation, key)?;
            if value.is_null() {
                if !nullable {
                    return Err(ResponseModelError::WrongType);
                }
                null_secrets
                    .try_reserve(1)
                    .map_err(|_| ResponseModelError::Allocation)?;
                null_secrets.push(key);
                continue;
            }
            let secret = value
                .take_string()
                .map(SensitiveText::new)
                .ok_or(ResponseModelError::WrongType)?;
            secret.validate(65_536)?;
            secrets
                .try_reserve(1)
                .map_err(|_| ResponseModelError::Allocation)?;
            secrets.push(NamedSensitiveText::new(key, secret));
        }
    }
    Ok((secrets, null_secrets))
}

fn secret_policy(operation: &str, key: &str) -> Result<bool, ResponseModelError> {
    match (operation, key) {
        ("create_server" | "rebuild_server", "root_password") => Ok(true),
        ("enable_server_rescue" | "reset_server_password", "root_password")
        | ("request_server_console", "password" | "wss_url") => Ok(false),
        _ => Err(ResponseModelError::EnvelopeMismatch),
    }
}

fn object_mut(value: &mut Value) -> Result<&mut Map, ResponseModelError> {
    value.as_object_mut().ok_or(ResponseModelError::WrongType)
}

fn validate_page_item_count(
    value: &Value,
    pagination: &crate::pagination::PaginationMetadata,
) -> Result<(), ResponseModelError> {
    let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
    if values.len() > usize::from(pagination.per_page().get()) {
        return Err(ResponseModelError::InvalidPagination);
    }
    Ok(())
}

fn map_json_error(error: strict_json::JsonError) -> HetznerDecodeError {
    match error {
        strict_json::JsonError::Allocation => {
            HetznerDecodeError::Model(ResponseModelError::Allocation)
        }
        _ => HetznerDecodeError::MalformedPayload,
    }
}

fn required_mut<'a>(object: &'a mut Map, key: &str) -> Result<&'a mut Value, ResponseModelError> {
    object.get_mut(key).ok_or(ResponseModelError::MissingField)
}

fn validate_required(envelope: &Map, required_fields: &str) -> Result<(), ResponseModelError> {
    if required_fields == "-" {
        return Ok(());
    }
    for field in required_fields.split(',') {
        if !envelope.contains_key(field) {
            return Err(ResponseModelError::MissingField);
        }
    }
    Ok(())
}
