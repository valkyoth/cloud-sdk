use core::cmp::Ordering;

use cloud_sdk_sanitization::{SecretBoxBytes, sanitize_value};

/// Maximum bytes in a standard-product or addon identifier.
pub const MAX_ROBOT_ORDER_PRODUCT_ID_BYTES: usize = 128;
/// Maximum bytes in a source-locked location selector.
pub const MAX_ROBOT_ORDER_LOCATION_BYTES: usize = 64;
/// Maximum bytes in a distribution, language, or addon selection.
pub const MAX_ROBOT_ORDER_CHOICE_BYTES: usize = 512;
/// Maximum bytes in a Robot ordering transaction identifier.
pub const MAX_ROBOT_ORDER_TRANSACTION_ID_BYTES: usize = 128;
const MAX_DECIMAL_BYTES: usize = 24;
const MAX_DECIMAL_DIGITS: usize = 18;
const MAX_DECIMAL_SCALE: u8 = 4;

/// Failure while admitting an exact Robot ordering value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotOrderValueError {
    /// A product identifier was empty, oversized, or outside its path profile.
    InvalidProductId,
    /// A Server Auction product identifier was zero.
    InvalidMarketProductId,
    /// A location was empty, oversized, or outside its selector profile.
    InvalidLocation,
    /// A selected catalog value was empty, oversized, or contained controls.
    InvalidChoice,
    /// A transaction identifier was empty, oversized, or outside its path profile.
    InvalidTransactionId,
    /// A decimal was negative, noncanonical, oversized, or too precise.
    InvalidDecimal,
    /// A currency was not a three-letter uppercase code.
    InvalidCurrency,
    /// Protected storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotOrderValueError,
    Self::InvalidProductId => "Robot order product identifier is invalid",
    Self::InvalidMarketProductId => "Robot market product identifier is invalid",
    Self::InvalidLocation => "Robot order location is invalid",
    Self::InvalidChoice => "Robot order catalog choice is invalid",
    Self::InvalidTransactionId => "Robot order transaction identifier is invalid",
    Self::InvalidDecimal => "Robot order decimal is invalid",
    Self::InvalidCurrency => "Robot order currency is invalid",
    Self::Allocation => "Robot order value allocation failed",
);

macro_rules! protected_text {
    ($name:ident, $maximum:expr, $variant:ident, $validate:ident, $description:literal) => {
        #[doc = $description]
        pub struct $name(SecretBoxBytes);

        impl $name {
            /// Copies one validated value into protected owned storage.
            pub fn new(value: &str) -> Result<Self, RobotOrderValueError> {
                if !$validate(value, $maximum) {
                    return Err(RobotOrderValueError::$variant);
                }
                SecretBoxBytes::try_from_slice(value.as_bytes(), $maximum)
                    .map(Self)
                    .map_err(|_| RobotOrderValueError::Allocation)
            }

            /// Runs a closure with temporary access to the exact value.
            pub fn try_with_text<R>(
                &self,
                inspect: impl FnOnce(&str) -> R,
            ) -> Result<R, core::str::Utf8Error> {
                self.0
                    .with_secret(|bytes| core::str::from_utf8(bytes).map(inspect))
            }

            #[allow(
                dead_code,
                reason = "some protected ordering values need internal text access only with Serde"
            )]
            pub(super) fn with_text<R>(&self, inspect: impl FnOnce(&str) -> R) -> R {
                self.0.with_secret(|bytes| {
                    let value = core::str::from_utf8(bytes)
                        .unwrap_or_else(|_| unreachable!("protected Robot order text lost UTF-8"));
                    inspect(value)
                })
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                other.0.with_secret(|right| self.0.constant_time_eq(right))
            }
        }
        impl Eq for $name {}
        impl core::fmt::Debug for $name {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(concat!(stringify!($name), "([redacted])"))
            }
        }
    };
}

protected_text!(
    RobotOrderProductId,
    MAX_ROBOT_ORDER_PRODUCT_ID_BYTES,
    InvalidProductId,
    valid_identifier,
    "Protected standard-server or addon product identifier."
);
protected_text!(
    RobotOrderLocation,
    MAX_ROBOT_ORDER_LOCATION_BYTES,
    InvalidLocation,
    valid_identifier,
    "Protected Robot catalog location selector."
);
protected_text!(
    RobotOrderChoice,
    MAX_ROBOT_ORDER_CHOICE_BYTES,
    InvalidChoice,
    valid_choice,
    "Protected distribution, language, or addon choice."
);
protected_text!(
    RobotOrderTransactionId,
    MAX_ROBOT_ORDER_TRANSACTION_ID_BYTES,
    InvalidTransactionId,
    valid_identifier,
    "Protected Robot ordering transaction identifier."
);

/// Non-zero Server Auction product identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RobotMarketProductId(u64);

impl RobotMarketProductId {
    /// Creates a non-zero Server Auction product identifier.
    pub const fn new(value: u64) -> Result<Self, RobotOrderValueError> {
        if value == 0 {
            Err(RobotOrderValueError::InvalidMarketProductId)
        } else {
            Ok(Self(value))
        }
    }

    /// Returns the provider identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Exact non-negative decimal retained without floating-point conversion.
pub struct RobotOrderDecimal {
    text: SecretBoxBytes,
    coefficient: u64,
    scale: u8,
}

impl RobotOrderDecimal {
    /// Parses a canonical decimal with at most four fractional digits.
    pub fn new(value: &str) -> Result<Self, RobotOrderValueError> {
        let (coefficient, scale) = parse_decimal(value)?;
        let text = SecretBoxBytes::try_from_slice(value.as_bytes(), MAX_DECIMAL_BYTES)
            .map_err(|_| RobotOrderValueError::Allocation)?;
        Ok(Self {
            text,
            coefficient,
            scale,
        })
    }

    /// Returns the number of retained fractional digits.
    #[must_use]
    pub const fn scale(&self) -> u8 {
        self.scale
    }

    /// Runs a closure with the exact provider decimal text.
    pub fn try_with_text<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.text
            .with_secret(|bytes| core::str::from_utf8(bytes).map(inspect))
    }

    pub(super) fn with_text<R>(&self, inspect: impl FnOnce(&str) -> R) -> R {
        self.text.with_secret(|bytes| {
            let value = core::str::from_utf8(bytes)
                .unwrap_or_else(|_| unreachable!("protected Robot decimal lost UTF-8"));
            inspect(value)
        })
    }

    fn normalized(&self, scale: u8) -> u128 {
        let exponent = scale.saturating_sub(self.scale);
        u128::from(self.coefficient).saturating_mul(pow10(exponent))
    }

    #[cfg(feature = "serde")]
    pub(in crate::robot::ordering) fn checked_units(&self, scale: u8) -> Option<u128> {
        (scale >= self.scale).then(|| self.normalized(scale))
    }
}

impl PartialEq for RobotOrderDecimal {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for RobotOrderDecimal {}
impl PartialOrd for RobotOrderDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for RobotOrderDecimal {
    fn cmp(&self, other: &Self) -> Ordering {
        let scale = self.scale.max(other.scale);
        self.normalized(scale).cmp(&other.normalized(scale))
    }
}
impl core::fmt::Debug for RobotOrderDecimal {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotOrderDecimal([redacted])")
    }
}

impl Drop for RobotOrderDecimal {
    fn drop(&mut self) {
        sanitize_value(&mut self.coefficient);
        sanitize_value(&mut self.scale);
    }
}

/// Protected three-letter account currency code.
pub struct RobotOrderCurrency(SecretBoxBytes);

impl RobotOrderCurrency {
    /// Creates an uppercase three-letter currency code.
    pub fn new(value: &str) -> Result<Self, RobotOrderValueError> {
        if value.len() != 3 || !value.bytes().all(|byte| byte.is_ascii_uppercase()) {
            return Err(RobotOrderValueError::InvalidCurrency);
        }
        SecretBoxBytes::try_from_slice(value.as_bytes(), 3)
            .map(Self)
            .map_err(|_| RobotOrderValueError::Allocation)
    }

    /// Runs a closure with the exact currency code.
    pub fn try_with_code<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0
            .with_secret(|bytes| core::str::from_utf8(bytes).map(inspect))
    }

    #[cfg(feature = "serde")]
    pub(in crate::robot::ordering) fn with_code<R>(&self, inspect: impl FnOnce(&str) -> R) -> R {
        self.0.with_secret(|bytes| {
            let code = core::str::from_utf8(bytes)
                .unwrap_or_else(|_| unreachable!("protected Robot currency lost UTF-8"));
            inspect(code)
        })
    }
}

impl PartialEq for RobotOrderCurrency {
    fn eq(&self, other: &Self) -> bool {
        other.0.with_secret(|right| self.0.constant_time_eq(right))
    }
}
impl Eq for RobotOrderCurrency {}
impl core::fmt::Debug for RobotOrderCurrency {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotOrderCurrency([redacted])")
    }
}

fn valid_identifier(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_choice(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.as_bytes().first() != Some(&b' ')
        && value.as_bytes().last() != Some(&b' ')
        && !value.bytes().any(|byte| byte.is_ascii_control())
}

fn parse_decimal(value: &str) -> Result<(u64, u8), RobotOrderValueError> {
    if value.is_empty() || value.len() > MAX_DECIMAL_BYTES || value.starts_with('+') {
        return Err(RobotOrderValueError::InvalidDecimal);
    }
    let mut split = value.split('.');
    let whole = split.next().unwrap_or_default();
    let fraction = split.next();
    if split.next().is_some()
        || whole.is_empty()
        || (whole.len() > 1 && whole.starts_with('0'))
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(RobotOrderValueError::InvalidDecimal);
    }
    let fraction = fraction.unwrap_or_default();
    if (value.contains('.') && fraction.is_empty())
        || fraction.len() > usize::from(MAX_DECIMAL_SCALE)
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        || whole.len().saturating_add(fraction.len()) > MAX_DECIMAL_DIGITS
    {
        return Err(RobotOrderValueError::InvalidDecimal);
    }
    let coefficient = whole
        .bytes()
        .chain(fraction.bytes())
        .try_fold(0_u64, |current, byte| {
            current
                .checked_mul(10)
                .and_then(|value| value.checked_add(u64::from(byte.saturating_sub(b'0'))))
        })
        .ok_or(RobotOrderValueError::InvalidDecimal)?;
    let scale = u8::try_from(fraction.len()).map_err(|_| RobotOrderValueError::InvalidDecimal)?;
    Ok((coefficient, scale))
}

const fn pow10(exponent: u8) -> u128 {
    let mut value = 1_u128;
    let mut current = 0_u8;
    while current < exponent {
        value = value.saturating_mul(10);
        current = current.saturating_add(1);
    }
    value
}
