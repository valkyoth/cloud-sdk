//! Metrics, pricing, folder, and sensitive text models.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use cloud_sdk_sanitization::SecretString;

use super::{ResponseModelError, checked_text, object, required, validate_text, value_text};
use crate::serde::strict_json::{Map, Value};

const MAX_FOLDERS: usize = 4096;
const MAX_METRIC_SERIES: usize = 512;
const MAX_METRIC_POINTS: usize = 65_536;

/// Sensitive provider text requiring closure-scoped access.
///
/// The owned allocation is cleared when this value is dropped. This type does
/// not implement `Clone`; callers must not duplicate protected response text
/// through an infallible allocation path.
///
/// ```compile_fail
/// use cloud_sdk_hetzner::serde::SensitiveText;
///
/// fn duplicate(secret: SensitiveText) {
///     let _ = secret.clone();
/// }
/// ```
pub struct SensitiveText(SecretString);

impl SensitiveText {
    pub(crate) fn new(value: SecretString) -> Self {
        Self(value)
    }

    /// Runs a closure with temporary read-only access to the sensitive text.
    pub fn try_with_secret<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0.try_with_secret(inspect)
    }

    pub(crate) fn validate(&self, max: usize) -> Result<(), ResponseModelError> {
        self.try_with_secret(|value| validate_text(value, max))
            .map_err(|_| ResponseModelError::InvalidText)?
    }

    pub(crate) fn validate_multiline(&self, max: usize) -> Result<(), ResponseModelError> {
        self.try_with_secret(|value| {
            if value.is_empty()
                || value.len() > max
                || value.chars().any(|character| {
                    !matches!(character, '\t' | '\n' | '\r')
                        && (character.is_control()
                            || matches!(
                                character,
                                '\u{061c}'
                                    | '\u{200b}'..='\u{200f}'
                                    | '\u{202a}'..='\u{202e}'
                                    | '\u{2060}'..='\u{2069}'
                                    | '\u{feff}'
                            ))
                })
            {
                Err(ResponseModelError::InvalidText)
            } else {
                Ok(())
            }
        })
        .map_err(|_| ResponseModelError::InvalidText)?
    }
}

impl PartialEq for SensitiveText {
    fn eq(&self, other: &Self) -> bool {
        other
            .try_with_secret(|value| self.0.constant_time_eq(value))
            .unwrap_or(false)
    }
}

impl Eq for SensitiveText {}

impl fmt::Debug for SensitiveText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SensitiveText([redacted])")
    }
}

/// Exported DNS zonefile with redacted diagnostics.
#[derive(Eq, PartialEq)]
pub struct ZoneFile(SensitiveText);

impl ZoneFile {
    /// Runs a closure with temporary access to the sensitive zonefile.
    pub fn try_with_zonefile<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0.try_with_secret(inspect)
    }
}

impl fmt::Debug for ZoneFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ZoneFile([redacted])")
    }
}

/// One timestamp/value pair from a metrics response.
#[derive(Clone, Debug, PartialEq)]
pub struct MetricPoint {
    timestamp: f64,
    value: String,
}

impl MetricPoint {
    /// Returns the provider timestamp.
    #[must_use]
    pub const fn timestamp(&self) -> f64 {
        self.timestamp
    }

    /// Returns the decimal metric value text without lossy conversion.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Named metrics time series.
#[derive(Clone, Debug, PartialEq)]
pub struct MetricSeries {
    name: String,
    points: Vec<MetricPoint>,
}

impl MetricSeries {
    /// Returns the provider series name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns bounded metric points.
    #[must_use]
    pub fn points(&self) -> &[MetricPoint] {
        &self.points
    }
}

/// Validated server or load-balancer metrics result.
#[derive(Clone, Debug, PartialEq)]
pub struct Metrics {
    start: String,
    end: String,
    step: f64,
    series: Vec<MetricSeries>,
}

impl Metrics {
    /// Returns the requested range start.
    #[must_use]
    pub fn start(&self) -> &str {
        &self.start
    }

    /// Returns the requested range end.
    #[must_use]
    pub fn end(&self) -> &str {
        &self.end
    }

    /// Returns the positive provider sampling step.
    #[must_use]
    pub const fn step(&self) -> f64 {
        self.step
    }

    /// Returns the bounded time series.
    #[must_use]
    pub fn series(&self) -> &[MetricSeries] {
        &self.series
    }
}

/// Validated pricing summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pricing {
    currency: String,
    vat_rate: String,
    server_type_prices: usize,
    load_balancer_type_prices: usize,
}

impl Pricing {
    /// Returns the provider currency code.
    #[must_use]
    pub fn currency(&self) -> &str {
        &self.currency
    }

    /// Returns the decimal VAT rate text.
    #[must_use]
    pub fn vat_rate(&self) -> &str {
        &self.vat_rate
    }

    /// Returns the number of server-type price records.
    #[must_use]
    pub const fn server_type_prices(&self) -> usize {
        self.server_type_prices
    }

    /// Returns the number of load-balancer-type price records.
    #[must_use]
    pub const fn load_balancer_type_prices(&self) -> usize {
        self.load_balancer_type_prices
    }
}

/// Bounded Storage Box folder paths.
#[derive(Clone, Eq, PartialEq)]
pub struct FolderList(Vec<String>);

impl FolderList {
    /// Returns validated folder paths.
    #[must_use]
    pub fn folders(&self) -> &[String] {
        &self.0
    }
}

impl fmt::Debug for FolderList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FolderList")
            .field("count", &self.0.len())
            .field("folders", &"[redacted]")
            .finish()
    }
}

pub(crate) fn parse_zonefile(value: &mut Value) -> Result<ZoneFile, ResponseModelError> {
    let secret = value
        .take_string()
        .map(SensitiveText::new)
        .ok_or(ResponseModelError::WrongType)?;
    secret.validate_multiline(8_388_608)?;
    Ok(ZoneFile(secret))
}

pub(crate) fn parse_folders(value: &Value) -> Result<FolderList, ResponseModelError> {
    let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
    if values.len() > MAX_FOLDERS {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut folders = Vec::new();
    folders
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        folders.push(value_text(value, 4096)?);
    }
    Ok(FolderList(folders))
}

pub(crate) fn parse_metrics(value: &Value) -> Result<Metrics, ResponseModelError> {
    let fields = object(value)?;
    let start = text(fields, "start", 64)?;
    let end = text(fields, "end", 64)?;
    let step = required(fields, "step")?
        .as_f64()
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or(ResponseModelError::InvalidNumber)?;
    let time_series = object(required(fields, "time_series")?)?;
    if time_series.len() > MAX_METRIC_SERIES {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut series = Vec::new();
    series
        .try_reserve_exact(time_series.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for (name, value) in time_series.iter() {
        let name = checked_text(name.as_str(), 256)?;
        let series_fields = object(value)?;
        let values = required(series_fields, "values")?
            .as_array()
            .ok_or(ResponseModelError::WrongType)?;
        if values.len() > MAX_METRIC_POINTS {
            return Err(ResponseModelError::TooManyItems);
        }
        let mut points = Vec::new();
        points
            .try_reserve_exact(values.len())
            .map_err(|_| ResponseModelError::Allocation)?;
        for point in values {
            let point = point.as_array().ok_or(ResponseModelError::WrongType)?;
            if point.len() != 2 {
                return Err(ResponseModelError::EnvelopeMismatch);
            }
            let timestamp = point
                .first()
                .and_then(Value::as_f64)
                .filter(|value| value.is_finite())
                .ok_or(ResponseModelError::InvalidNumber)?;
            let value = point
                .get(1)
                .ok_or(ResponseModelError::WrongType)
                .and_then(|value| value_text(value, 256))?;
            points.push(MetricPoint { timestamp, value });
        }
        series.push(MetricSeries { name, points });
    }
    Ok(Metrics {
        start,
        end,
        step,
        series,
    })
}

pub(crate) fn parse_pricing(value: &Value) -> Result<Pricing, ResponseModelError> {
    let fields = object(value)?;
    for key in [
        "primary_ips",
        "floating_ips",
        "server_types",
        "load_balancer_types",
    ] {
        required(fields, key)?
            .as_array()
            .ok_or(ResponseModelError::WrongType)?;
    }
    for key in ["image", "volume", "server_backup", "floating_ip"] {
        object(required(fields, key)?)?;
    }
    let server_type_prices = required(fields, "server_types")?
        .as_array()
        .ok_or(ResponseModelError::WrongType)?
        .len();
    let load_balancer_type_prices = required(fields, "load_balancer_types")?
        .as_array()
        .ok_or(ResponseModelError::WrongType)?
        .len();
    Ok(Pricing {
        currency: text(fields, "currency", 16)?,
        vat_rate: text(fields, "vat_rate", 64)?,
        server_type_prices,
        load_balancer_type_prices,
    })
}

fn text(fields: &Map, key: &str, max: usize) -> Result<String, ResponseModelError> {
    value_text(required(fields, key)?, max)
}

#[cfg(test)]
mod tests {
    use cloud_sdk_sanitization::SecretString;

    use super::SensitiveText;

    #[test]
    fn sensitive_text_adopts_protected_storage_without_another_allocation() {
        let protected = SecretString::from_secret_str("temporary secret");
        let before = protected.with_secret_bytes(|value| value.as_ptr());
        let secret = SensitiveText::new(protected);
        let after = secret.0.with_secret_bytes(|value| value.as_ptr());

        assert_eq!(before, after);
        assert_eq!(
            secret.try_with_secret(|value| value == "temporary secret"),
            Ok(true)
        );
        assert!(!alloc::format!("{secret:?}").contains("temporary secret"));
    }

    #[test]
    fn sensitive_text_equality_compares_secret_contents() {
        let left = SensitiveText::new(SecretString::from_secret_str("same"));
        let equal = SensitiveText::new(SecretString::from_secret_str("same"));
        let different = SensitiveText::new(SecretString::from_secret_str("different"));

        assert_eq!(left, equal);
        assert_ne!(left, different);
    }
}
