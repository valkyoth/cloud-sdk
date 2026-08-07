use super::*;

pub(super) fn decode_checked_success(
    operation: &str,
    binding: ResponseBinding,
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
    quota: Box<HetznerQuota>,
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
        if operation == "list_storage_boxes" {
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
    quota: Box<HetznerQuota>,
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
        return decode_composite(binding, object_mut(value)?);
    }
    if binding.shape == ResponseShape::ZoneFile {
        let envelope = object_mut(value)?;
        return parse_zonefile(required_mut(envelope, "zonefile")?).map(HetznerSuccess::ZoneFile);
    }
    match operation {
        "list_locations" => return parse_location_page(value).map(HetznerSuccess::Locations),
        "get_certificate" => return parse_certificate(value).map(HetznerSuccess::Certificate),
        "list_storage_boxes" => {
            return parse_storage_box_page(value).map(HetznerSuccess::StorageBoxes);
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
            decode_resources(binding, object(value)?)
        }
        ResponseShape::Composite => Err(ResponseModelError::EnvelopeMismatch),
        ResponseShape::Metrics => {
            parse_metrics(required(object(value)?, "metrics")?).map(HetznerSuccess::Metrics)
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
    envelope: &Map,
) -> Result<HetznerSuccess, ResponseModelError> {
    let value = required(envelope, binding.root)?;
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
    binding: ResponseBinding,
    envelope: &mut Map,
) -> Result<HetznerSuccess, ResponseModelError> {
    let secrets = take_composite_secrets(envelope)?;
    let resource = if binding.root == "-" {
        None
    } else {
        envelope
            .get(binding.root)
            .map(|value| parse_resource(binding.root, value))
            .transpose()?
    };
    let mut actions = Vec::new();
    if let Some(value) = envelope.get_mut("action") {
        actions
            .try_reserve(1)
            .map_err(|_| ResponseModelError::Allocation)?;
        actions.push(parse_action(value)?);
    }
    for key in ["actions", "next_actions"] {
        if let Some(value) = envelope.get_mut(key) {
            let parsed = parse_actions(value)?;
            actions
                .try_reserve(parsed.len())
                .map_err(|_| ResponseModelError::Allocation)?;
            actions.extend(parsed);
        }
    }
    Ok(HetznerSuccess::Composite(CompositeResult {
        resource,
        actions,
        secrets,
    }))
}

fn take_composite_secrets(
    envelope: &mut Map,
) -> Result<Vec<NamedSensitiveText>, ResponseModelError> {
    let mut secrets = Vec::new();
    for key in ["root_password", "password", "wss_url"] {
        if let Some(value) = envelope.get_mut(key) {
            if value.is_null() {
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
    Ok(secrets)
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
