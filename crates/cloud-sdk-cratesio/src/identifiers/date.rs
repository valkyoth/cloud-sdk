use super::IdentifierError;

/// Canonical Gregorian `YYYY-MM-DD` in years 0001 through 9999.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Date<'a>(&'a str);
impl<'a> Date<'a> {
    /// Validates actual calendar days, including leap years.
    pub fn new(value: &'a str) -> Result<Self, IdentifierError> {
        let mut parts = value.split('-');
        let year = parts.next().ok_or(IdentifierError::Syntax)?;
        let month = parts.next().ok_or(IdentifierError::Syntax)?;
        let day = parts.next().ok_or(IdentifierError::Syntax)?;
        if parts.next().is_some()
            || year.len() != 4
            || month.len() != 2
            || day.len() != 2
            || !year
                .bytes()
                .chain(month.bytes())
                .chain(day.bytes())
                .all(|b| b.is_ascii_digit())
        {
            return Err(IdentifierError::Syntax);
        }
        let y: u32 = year.parse().map_err(|_| IdentifierError::Syntax)?;
        let m: u32 = month.parse().map_err(|_| IdentifierError::Syntax)?;
        let d: u32 = day.parse().map_err(|_| IdentifierError::Syntax)?;
        let maximum = match m {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if y.is_multiple_of(400) || y.is_multiple_of(4) && !y.is_multiple_of(100) => 29,
            2 => 28,
            _ => return Err(IdentifierError::Syntax),
        };
        if y == 0 || d == 0 || d > maximum {
            return Err(IdentifierError::Syntax);
        }
        Ok(Self(value))
    }
    /// Returns canonical wire text.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}
