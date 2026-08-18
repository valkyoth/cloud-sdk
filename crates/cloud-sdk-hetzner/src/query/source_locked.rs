//! Source-derived fallback for complete OpenAPI query execution.

use cloud_sdk::buffer;
use cloud_sdk::transport::MAX_REQUEST_TARGET_BYTES;

use super::MAX_QUERY_VALUE_BYTES;

const CONTRACTS: &str = include_str!("source_contracts.tsv");
const MAX_ARGUMENTS: usize = 128;

mod validation;

/// A query-bearing operation from the pinned Hetzner schemas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceQueryOperation(&'static str);

impl SourceQueryOperation {
    /// Returns the source operation ID.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

include!("source_operations.rs");

/// Query parameter names present in the pinned schemas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceQueryParameter {
    /// `architecture`.
    Architecture,
    /// `bound_to`.
    BoundTo,
    /// `end`.
    End,
    /// `fingerprint`.
    Fingerprint,
    /// `id`.
    Id,
    /// `include_architecture_wildcard`.
    IncludeArchitectureWildcard,
    /// `include_deprecated`.
    IncludeDeprecated,
    /// `ip`.
    Ip,
    /// `is_automatic`.
    IsAutomatic,
    /// `label_selector`.
    LabelSelector,
    /// `mode`.
    Mode,
    /// `name`.
    Name,
    /// `page`.
    Page,
    /// `path`.
    Path,
    /// `per_page`.
    PerPage,
    /// `sort`.
    Sort,
    /// `start`.
    Start,
    /// `status`.
    Status,
    /// `step`.
    Step,
    /// `type`.
    Type,
    /// `username`.
    Username,
}

impl SourceQueryParameter {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Architecture => "architecture",
            Self::BoundTo => "bound_to",
            Self::End => "end",
            Self::Fingerprint => "fingerprint",
            Self::Id => "id",
            Self::IncludeArchitectureWildcard => "include_architecture_wildcard",
            Self::IncludeDeprecated => "include_deprecated",
            Self::Ip => "ip",
            Self::IsAutomatic => "is_automatic",
            Self::LabelSelector => "label_selector",
            Self::Mode => "mode",
            Self::Name => "name",
            Self::Page => "page",
            Self::Path => "path",
            Self::PerPage => "per_page",
            Self::Sort => "sort",
            Self::Start => "start",
            Self::Status => "status",
            Self::Step => "step",
            Self::Type => "type",
            Self::Username => "username",
        }
    }
}

/// Bounded text accepted by source-derived query contracts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceQueryText<'a>(&'a str);

impl<'a> SourceQueryText<'a> {
    /// Creates nonempty bounded text without control bytes.
    pub fn new(value: &'a str) -> Result<Self, SourceQueryError> {
        if value.is_empty()
            || value.len() > MAX_QUERY_VALUE_BYTES
            || value
                .chars()
                .any(crate::display::is_unsafe_display_character)
        {
            return Err(SourceQueryError::InvalidText);
        }
        Ok(Self(value))
    }

    /// Returns the validated value.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// Typed source-query value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceQueryValue<'a> {
    /// Unsigned integer parameter.
    Integer(u64),
    /// Boolean parameter.
    Boolean(bool),
    /// String parameter.
    Text(SourceQueryText<'a>),
}

/// One typed source-query argument.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceQueryArgument<'a> {
    parameter: SourceQueryParameter,
    value: SourceQueryValue<'a>,
}

impl<'a> SourceQueryArgument<'a> {
    /// Creates an integer argument.
    #[must_use]
    pub const fn integer(parameter: SourceQueryParameter, value: u64) -> Self {
        Self {
            parameter,
            value: SourceQueryValue::Integer(value),
        }
    }

    /// Creates a Boolean argument.
    #[must_use]
    pub const fn boolean(parameter: SourceQueryParameter, value: bool) -> Self {
        Self {
            parameter,
            value: SourceQueryValue::Boolean(value),
        }
    }

    /// Creates a text argument.
    #[must_use]
    pub const fn text(parameter: SourceQueryParameter, value: SourceQueryText<'a>) -> Self {
        Self {
            parameter,
            value: SourceQueryValue::Text(value),
        }
    }
}

/// Source-query validation or encoding error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceQueryError {
    /// The global argument bound was exceeded.
    TooManyArguments,
    /// The parameter is not declared by the operation.
    UnknownParameter,
    /// The value type differs from the source schema.
    WrongValueKind,
    /// A required query parameter is absent.
    MissingRequiredParameter,
    /// A scalar parameter was supplied more than once.
    DuplicateScalar,
    /// One array value was supplied more than once.
    DuplicateArrayValue,
    /// An integer is zero or exceeds the safe API bound.
    InvalidInteger,
    /// Text is empty, too long, or contains controls.
    InvalidText,
    /// A string is outside the source enum.
    InvalidEnumValue,
    /// A metrics timestamp is malformed.
    InvalidTimestamp,
    /// A metrics step is not a positive `u32`.
    InvalidStep,
    /// The caller-owned output is too small.
    QueryBufferTooSmall,
    /// The embedded generated contract is malformed.
    CorruptContract,
}

impl_static_error!(SourceQueryError,
    Self::TooManyArguments => "source query has too many arguments",
    Self::UnknownParameter => "query parameter is not valid for the operation",
    Self::WrongValueKind => "query parameter value has the wrong type",
    Self::MissingRequiredParameter => "required query parameter is missing",
    Self::DuplicateScalar => "scalar query parameter is duplicated",
    Self::DuplicateArrayValue => "array query parameter value is duplicated",
    Self::InvalidInteger => "query integer is outside the admitted range",
    Self::InvalidText => "query text is invalid",
    Self::InvalidEnumValue => "query enum value is not source-locked",
    Self::InvalidTimestamp => "query timestamp is invalid",
    Self::InvalidStep => "query metrics step is invalid",
    Self::QueryBufferTooSmall => "source query output buffer is too small",
    Self::CorruptContract => "embedded source query contract is corrupt",
);

/// Fully validated query tied to one source operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceLockedQuery<'a> {
    operation: SourceQueryOperation,
    arguments: &'a [SourceQueryArgument<'a>],
}

impl<'a> SourceLockedQuery<'a> {
    /// Validates ownership, cardinality, required fields, enums, and bounds.
    pub fn try_new(
        operation: SourceQueryOperation,
        arguments: &'a [SourceQueryArgument<'a>],
    ) -> Result<Self, SourceQueryError> {
        if arguments.len() > MAX_ARGUMENTS {
            return Err(SourceQueryError::TooManyArguments);
        }
        let mut operation_found = false;
        for line in CONTRACTS.lines().skip(1) {
            let contract = Contract::parse(line)?;
            if contract.operation != operation.as_str() {
                continue;
            }
            operation_found = true;
            let count = validation::validate_contract_arguments(contract, arguments)?;
            if contract.required && count == 0 {
                return Err(SourceQueryError::MissingRequiredParameter);
            }
        }
        if !operation_found {
            return Err(SourceQueryError::CorruptContract);
        }
        for argument in arguments {
            if find_contract(operation, argument.parameter)?.is_none() {
                return Err(SourceQueryError::UnknownParameter);
            }
        }
        validation::validate_cross_fields(operation, arguments)?;
        Ok(Self {
            operation,
            arguments,
        })
    }

    /// Returns the source operation ID.
    #[must_use]
    pub const fn operation_id(self) -> &'static str {
        self.operation.as_str()
    }

    /// Writes canonical form-exploded query parameters atomically.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, SourceQueryError> {
        buffer::encode_snapshot_bounded(
            self,
            output,
            MAX_REQUEST_TARGET_BYTES,
            SourceQueryError::QueryBufferTooSmall,
            |query, encoder| {
                let mut first = true;
                for line in CONTRACTS.lines().skip(1) {
                    let contract = Contract::parse(line)?;
                    if contract.operation != query.operation.as_str() {
                        continue;
                    }
                    if contract.comma_separated() {
                        write_comma_arguments(encoder, &mut first, contract, query.arguments)?;
                        continue;
                    }
                    for argument in query.arguments {
                        if argument.parameter.as_str() == contract.name {
                            write_argument(encoder, &mut first, *argument)?;
                        }
                    }
                }
                Ok(())
            },
        )
    }
}

#[derive(Clone, Copy)]
struct Contract<'a> {
    operation: &'a str,
    name: &'a str,
    required: bool,
    kind: &'a str,
    allowed: &'a str,
    encoding: &'a str,
}

impl<'a> Contract<'a> {
    fn parse(line: &'a str) -> Result<Self, SourceQueryError> {
        let mut fields = line.split('\t');
        let contract = Self {
            operation: fields.next().ok_or(SourceQueryError::CorruptContract)?,
            name: fields.next().ok_or(SourceQueryError::CorruptContract)?,
            required: match fields.next() {
                Some("yes") => true,
                Some("no") => false,
                _ => return Err(SourceQueryError::CorruptContract),
            },
            kind: fields.next().ok_or(SourceQueryError::CorruptContract)?,
            encoding: fields.next().ok_or(SourceQueryError::CorruptContract)?,
            allowed: fields.next().ok_or(SourceQueryError::CorruptContract)?,
        };
        let fingerprint = fields.next().ok_or(SourceQueryError::CorruptContract)?;
        if fields.next().is_some() || fingerprint.len() != 64 {
            return Err(SourceQueryError::CorruptContract);
        }
        let expected_encoding = if contract.repeated() {
            "repeat"
        } else {
            "scalar"
        };
        if contract.encoding != expected_encoding
            && !(contract.repeated() && contract.encoding == "comma")
        {
            return Err(SourceQueryError::CorruptContract);
        }
        Ok(contract)
    }

    fn repeated(self) -> bool {
        self.kind.ends_with("[]")
    }

    fn comma_separated(self) -> bool {
        self.encoding == "comma"
    }
}

fn find_contract(
    operation: SourceQueryOperation,
    parameter: SourceQueryParameter,
) -> Result<Option<Contract<'static>>, SourceQueryError> {
    for line in CONTRACTS.lines().skip(1) {
        let contract = Contract::parse(line)?;
        if contract.operation == operation.as_str() && contract.name == parameter.as_str() {
            return Ok(Some(contract));
        }
    }
    Ok(None)
}

fn write_argument(
    encoder: &mut buffer::SnapshotEncoder<'_, SourceQueryError>,
    first: &mut bool,
    argument: SourceQueryArgument<'_>,
) -> Result<(), SourceQueryError> {
    let name = argument.parameter.as_str();
    match argument.value {
        SourceQueryValue::Integer(value) => encoder.query_u64(first, name, value),
        SourceQueryValue::Boolean(value) => {
            encoder.query_pair(first, name, if value { "true" } else { "false" })
        }
        SourceQueryValue::Text(value) => encoder.query_pair(first, name, value.as_str()),
    }
}

fn write_comma_arguments(
    encoder: &mut buffer::SnapshotEncoder<'_, SourceQueryError>,
    first: &mut bool,
    contract: Contract<'_>,
    arguments: &[SourceQueryArgument<'_>],
) -> Result<(), SourceQueryError> {
    let mut values = arguments
        .iter()
        .filter(|argument| argument.parameter.as_str() == contract.name);
    let Some(initial) = values.next() else {
        return Ok(());
    };
    encoder.query_separator(first)?;
    encoder.string(contract.name)?;
    encoder.string("=")?;
    let SourceQueryValue::Text(initial) = initial.value else {
        return Err(SourceQueryError::WrongValueKind);
    };
    encoder.percent_encoded(initial.as_str())?;
    for value in values {
        let SourceQueryValue::Text(value) = value.value else {
            return Err(SourceQueryError::WrongValueKind);
        };
        encoder.percent_encoded(",")?;
        encoder.percent_encoded(value.as_str())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
