use cloud_sdk_sanitization::SecretBoxBytes;

const MAX_INTERVAL_BYTES: usize = 13;

/// Traffic aggregation and interval grammar selected by Robot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotTrafficGranularity {
    /// `YYYY-MM-DDTHH` bounds and hourly values.
    Day,
    /// `YYYY-MM-DD` bounds and daily values.
    Month,
    /// `YYYY-MM` bounds and monthly values.
    Year,
}

impl RobotTrafficGranularity {
    pub(super) const fn wire_name(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Month => "month",
            Self::Year => "year",
        }
    }
}

/// Failure while constructing a Robot traffic interval.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotTrafficIntervalError {
    /// A bound did not match the selected source grammar or component ranges.
    InvalidBound,
    /// The lower bound sorts after the upper bound.
    Reversed,
    /// Protected interval storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotTrafficIntervalError,
    Self::InvalidBound => "Robot traffic interval bound is invalid",
    Self::Reversed => "Robot traffic interval is reversed",
    Self::Allocation => "Robot traffic interval allocation failed",
);

/// Protected exact request interval for one Robot traffic query.
pub struct RobotTrafficInterval {
    granularity: RobotTrafficGranularity,
    from: SecretBoxBytes,
    to: SecretBoxBytes,
}

impl RobotTrafficInterval {
    /// Creates an exact source-compatible interval.
    ///
    /// Components are range-checked, but month-length rules are deliberately
    /// not imposed because Hetzner's published month example uses day `31`
    /// for September. Lexical ordering is chronological for all three forms.
    pub fn new(
        granularity: RobotTrafficGranularity,
        from: &str,
        to: &str,
    ) -> Result<Self, RobotTrafficIntervalError> {
        if !valid_bound(granularity, from) || !valid_bound(granularity, to) {
            return Err(RobotTrafficIntervalError::InvalidBound);
        }
        if from > to {
            return Err(RobotTrafficIntervalError::Reversed);
        }
        let from = SecretBoxBytes::try_from_slice(from.as_bytes(), MAX_INTERVAL_BYTES)
            .map_err(|_| RobotTrafficIntervalError::Allocation)?;
        let to = SecretBoxBytes::try_from_slice(to.as_bytes(), MAX_INTERVAL_BYTES)
            .map_err(|_| RobotTrafficIntervalError::Allocation)?;
        Ok(Self {
            granularity,
            from,
            to,
        })
    }

    /// Returns the selected aggregation granularity.
    #[must_use]
    pub const fn granularity(&self) -> RobotTrafficGranularity {
        self.granularity
    }

    /// Runs a closure with the exact lower bound.
    pub fn with_from<R>(&self, inspect: impl FnOnce(&str) -> R) -> R {
        with_text(&self.from, inspect)
    }

    /// Runs a closure with the exact upper bound.
    pub fn with_to<R>(&self, inspect: impl FnOnce(&str) -> R) -> R {
        with_text(&self.to, inspect)
    }

    pub(super) fn matches_from(&self, value: &str) -> bool {
        self.with_from(|expected| expected == value)
    }

    pub(super) fn matches_to(&self, value: &str) -> bool {
        self.with_to(|expected| expected == value)
    }
}

impl core::fmt::Debug for RobotTrafficInterval {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotTrafficInterval")
            .field("granularity", &self.granularity)
            .field("bounds", &"[redacted]")
            .finish()
    }
}

fn with_text<R>(value: &SecretBoxBytes, inspect: impl FnOnce(&str) -> R) -> R {
    value.with_secret(|bytes| {
        let text = core::str::from_utf8(bytes)
            .unwrap_or_else(|_| unreachable!("protected traffic interval lost UTF-8"));
        inspect(text)
    })
}

fn valid_bound(granularity: RobotTrafficGranularity, value: &str) -> bool {
    let bytes = value.as_bytes();
    let expected = match granularity {
        RobotTrafficGranularity::Day => 13,
        RobotTrafficGranularity::Month => 10,
        RobotTrafficGranularity::Year => 7,
    };
    if bytes.len() != expected
        || bytes.get(0..4) == Some(b"0000")
        || bytes.get(4) != Some(&b'-')
        || bytes
            .get(7)
            .is_some_and(|byte| expected > 7 && *byte != b'-')
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| !matches!(index, 4 | 7 | 10) && !byte.is_ascii_digit())
    {
        return false;
    }
    if matches!(granularity, RobotTrafficGranularity::Day) && bytes.get(10) != Some(&b'T') {
        return false;
    }
    let month = decimal(bytes, 5, 7);
    let day = decimal(bytes, 8, 10);
    let hour = decimal(bytes, 11, 13);
    matches!(month, Some(1..=12))
        && match granularity {
            RobotTrafficGranularity::Year => true,
            RobotTrafficGranularity::Month => matches!(day, Some(1..=31)),
            RobotTrafficGranularity::Day => {
                matches!(day, Some(1..=31)) && matches!(hour, Some(0..=23))
            }
        }
}

fn decimal(bytes: &[u8], start: usize, end: usize) -> Option<u8> {
    let tens = bytes.get(start)?.checked_sub(b'0')?;
    let ones = bytes.get(start.checked_add(1)?)?.checked_sub(b'0')?;
    (end == start.checked_add(2)? && tens <= 9 && ones <= 9)
        .then(|| tens.saturating_mul(10).saturating_add(ones))
}
