use super::{
    Contract, SourceQueryArgument, SourceQueryError, SourceQueryOperation, SourceQueryParameter,
    SourceQueryText, SourceQueryValue,
};

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

pub(super) fn validate_contract_arguments(
    contract: Contract<'_>,
    arguments: &[SourceQueryArgument<'_>],
) -> Result<usize, SourceQueryError> {
    let mut count = 0_usize;
    for (index, argument) in arguments.iter().enumerate() {
        if argument.parameter.as_str() != contract.name {
            continue;
        }
        count = count
            .checked_add(1)
            .ok_or(SourceQueryError::TooManyArguments)?;
        validate_value(contract, argument.value)?;
        if !contract.repeated() && count > 1 {
            return Err(SourceQueryError::DuplicateScalar);
        }
        let previous_arguments = arguments
            .get(..index)
            .ok_or(SourceQueryError::CorruptContract)?;
        if contract.repeated()
            && previous_arguments.iter().any(|previous| {
                previous.parameter == argument.parameter && previous.value == argument.value
            })
        {
            return Err(SourceQueryError::DuplicateArrayValue);
        }
    }
    Ok(count)
}

fn validate_value(
    contract: Contract<'_>,
    value: SourceQueryValue<'_>,
) -> Result<(), SourceQueryError> {
    match (contract.kind.trim_end_matches("[]"), value) {
        ("integer", SourceQueryValue::Integer(value)) => {
            if value == 0 || value > MAX_SAFE_INTEGER {
                return Err(SourceQueryError::InvalidInteger);
            }
            if contract.name == "per_page" && value > u64::from(crate::pagination::MAX_PER_PAGE) {
                return Err(SourceQueryError::InvalidInteger);
            }
        }
        ("boolean", SourceQueryValue::Boolean(_)) => {}
        ("string", SourceQueryValue::Text(value)) => validate_text_contract(contract, value)?,
        _ => return Err(SourceQueryError::WrongValueKind),
    }
    Ok(())
}

fn validate_text_contract(
    contract: Contract<'_>,
    value: SourceQueryText<'_>,
) -> Result<(), SourceQueryError> {
    let value = value.as_str();
    if !contract.allowed.is_empty() && !contract.allowed.split('|').any(|item| item == value) {
        return Err(SourceQueryError::InvalidEnumValue);
    }
    if matches!(contract.name, "start" | "end") && !valid_timestamp(value) {
        return Err(SourceQueryError::InvalidTimestamp);
    }
    if contract.name == "step"
        && value
            .parse::<u32>()
            .ok()
            .filter(|step| *step != 0)
            .is_none()
    {
        return Err(SourceQueryError::InvalidStep);
    }
    Ok(())
}

pub(super) fn validate_cross_fields(
    operation: SourceQueryOperation,
    arguments: &[SourceQueryArgument<'_>],
) -> Result<(), SourceQueryError> {
    if operation != SourceQueryOperation::GET_SERVER_METRICS
        && operation != SourceQueryOperation::GET_LOAD_BALANCER_METRICS
    {
        return Ok(());
    }
    let start = argument_text(arguments, SourceQueryParameter::Start)
        .ok_or(SourceQueryError::MissingRequiredParameter)?;
    let end = argument_text(arguments, SourceQueryParameter::End)
        .ok_or(SourceQueryError::MissingRequiredParameter)?;
    if start >= end {
        return Err(SourceQueryError::InvalidTimestamp);
    }
    Ok(())
}

fn argument_text<'a>(
    arguments: &'a [SourceQueryArgument<'a>],
    parameter: SourceQueryParameter,
) -> Option<&'a str> {
    arguments.iter().find_map(|argument| {
        if argument.parameter != parameter {
            return None;
        }
        match argument.value {
            SourceQueryValue::Text(value) => Some(value.as_str()),
            _ => None,
        }
    })
}

fn valid_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes.get(10) != Some(&b'T')
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
        || bytes.get(19) != Some(&b'Z')
    {
        return false;
    }
    let Some(year) = decimal(bytes, 0, 4) else {
        return false;
    };
    let Some(month) = decimal(bytes, 5, 7) else {
        return false;
    };
    let Some(day) = decimal(bytes, 8, 10) else {
        return false;
    };
    let Some(hour) = decimal(bytes, 11, 13) else {
        return false;
    };
    let Some(minute) = decimal(bytes, 14, 16) else {
        return false;
    };
    let Some(second) = decimal(bytes, 17, 19) else {
        return false;
    };
    month != 0
        && month <= 12
        && day != 0
        && day <= month_days(year, month)
        && hour <= 23
        && minute <= 59
        && second <= 59
}

fn decimal(bytes: &[u8], start: usize, end: usize) -> Option<u16> {
    let mut value = 0_u16;
    for byte in bytes.get(start..end)? {
        value = value
            .checked_mul(10)?
            .checked_add(u16::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}

const fn month_days(year: u16, month: u16) -> u16 {
    match month {
        2 if year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100)) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
