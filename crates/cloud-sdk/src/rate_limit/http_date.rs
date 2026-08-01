use super::{HttpDate, RetryAfterError, WallClockTimestamp};

pub(super) fn parse_http_date(
    value: &[u8],
    now: WallClockTimestamp,
) -> Result<HttpDate, RetryAfterError> {
    let parts = if value.len() == 29 && value.get(3) == Some(&b',') {
        parse_imf_fixdate(value)?
    } else if value.contains(&b',') {
        parse_rfc850(value, now)?
    } else {
        parse_asctime(value)?
    };
    parts.into_http_date()
}

#[derive(Clone, Copy)]
struct DateParts {
    weekday: u8,
    year: i64,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl DateParts {
    fn into_http_date(self) -> Result<HttpDate, RetryAfterError> {
        if self.year < 1601
            || self.month == 0
            || self.month > 12
            || self.day == 0
            || self.day > days_in_month(self.year, self.month)
            || self.hour > 23
            || self.minute > 59
            || self.second > 60
        {
            return Err(RetryAfterError::InvalidDate);
        }
        let days = days_from_civil(self.year, self.month, self.day)?;
        let weekday_source = days.checked_add(4).ok_or(RetryAfterError::Overflow)?;
        let weekday =
            u8::try_from(weekday_source.rem_euclid(7)).map_err(|_| RetryAfterError::Overflow)?;
        if weekday != self.weekday {
            return Err(RetryAfterError::WeekdayMismatch);
        }
        let seconds = days
            .checked_mul(86_400)
            .and_then(|value| {
                i64::from(self.hour)
                    .checked_mul(3_600)
                    .and_then(|part| value.checked_add(part))
            })
            .and_then(|value| {
                i64::from(self.minute)
                    .checked_mul(60)
                    .and_then(|part| value.checked_add(part))
            })
            .and_then(|value| value.checked_add(i64::from(self.second)))
            .ok_or(RetryAfterError::Overflow)?;
        Ok(HttpDate::new(seconds))
    }
}

fn parse_imf_fixdate(value: &[u8]) -> Result<DateParts, RetryAfterError> {
    if value.get(4) != Some(&b' ')
        || value.get(7) != Some(&b' ')
        || value.get(11) != Some(&b' ')
        || value.get(16) != Some(&b' ')
        || value.get(19) != Some(&b':')
        || value.get(22) != Some(&b':')
        || value.get(25..29) != Some(b" GMT")
    {
        return Err(RetryAfterError::InvalidSyntax);
    }
    Ok(DateParts {
        weekday: short_weekday(slice(value, 0, 3)?)?,
        day: number_u8(slice(value, 5, 7)?)?,
        month: month(slice(value, 8, 11)?)?,
        year: i64::from(number_u16(slice(value, 12, 16)?)?),
        hour: number_u8(slice(value, 17, 19)?)?,
        minute: number_u8(slice(value, 20, 22)?)?,
        second: number_u8(slice(value, 23, 25)?)?,
    })
}

fn parse_rfc850(value: &[u8], now: WallClockTimestamp) -> Result<DateParts, RetryAfterError> {
    let comma = value
        .iter()
        .position(|byte| *byte == b',')
        .ok_or(RetryAfterError::InvalidSyntax)?;
    let weekday = long_weekday(slice(value, 0, comma)?)?;
    let rest = value
        .get(comma.checked_add(2).ok_or(RetryAfterError::Overflow)?..)
        .ok_or(RetryAfterError::InvalidSyntax)?;
    if rest.len() != 22
        || rest.get(2) != Some(&b'-')
        || rest.get(6) != Some(&b'-')
        || rest.get(9) != Some(&b' ')
        || rest.get(12) != Some(&b':')
        || rest.get(15) != Some(&b':')
        || rest.get(18..22) != Some(b" GMT")
    {
        return Err(RetryAfterError::InvalidSyntax);
    }
    let day = number_u8(slice(rest, 0, 2)?)?;
    let month = month(slice(rest, 3, 6)?)?;
    let short_year = i64::from(number_u8(slice(rest, 7, 9)?)?);
    let hour = number_u8(slice(rest, 10, 12)?)?;
    let minute = number_u8(slice(rest, 13, 15)?)?;
    let second = number_u8(slice(rest, 16, 18)?)?;
    let current = civil_time_at(now)?;
    let mut year = current
        .year
        .checked_div(100)
        .and_then(|century| century.checked_mul(100))
        .and_then(|century| century.checked_add(short_year))
        .ok_or(RetryAfterError::Overflow)?;
    let cutoff_year = current
        .year
        .checked_add(50)
        .ok_or(RetryAfterError::Overflow)?;
    if (year, month, day, hour, minute, second)
        > (
            cutoff_year,
            current.month,
            current.day,
            current.hour,
            current.minute,
            current.second,
        )
    {
        year = year.checked_sub(100).ok_or(RetryAfterError::Overflow)?;
    }
    Ok(DateParts {
        weekday,
        day,
        month,
        year,
        hour,
        minute,
        second,
    })
}

fn parse_asctime(value: &[u8]) -> Result<DateParts, RetryAfterError> {
    if value.len() != 24
        || value.get(3) != Some(&b' ')
        || value.get(7) != Some(&b' ')
        || value.get(10) != Some(&b' ')
        || value.get(13) != Some(&b':')
        || value.get(16) != Some(&b':')
        || value.get(19) != Some(&b' ')
    {
        return Err(RetryAfterError::InvalidSyntax);
    }
    let day = match value.get(8) {
        Some(b' ') => number_u8(slice(value, 9, 10)?)?,
        Some(_) => number_u8(slice(value, 8, 10)?)?,
        None => return Err(RetryAfterError::InvalidSyntax),
    };
    Ok(DateParts {
        weekday: short_weekday(slice(value, 0, 3)?)?,
        month: month(slice(value, 4, 7)?)?,
        day,
        hour: number_u8(slice(value, 11, 13)?)?,
        minute: number_u8(slice(value, 14, 16)?)?,
        second: number_u8(slice(value, 17, 19)?)?,
        year: i64::from(number_u16(slice(value, 20, 24)?)?),
    })
}

fn slice(value: &[u8], start: usize, end: usize) -> Result<&[u8], RetryAfterError> {
    value.get(start..end).ok_or(RetryAfterError::InvalidSyntax)
}

fn number_u8(value: &[u8]) -> Result<u8, RetryAfterError> {
    u8::try_from(number_u16(value)?).map_err(|_| RetryAfterError::Overflow)
}

fn number_u16(value: &[u8]) -> Result<u16, RetryAfterError> {
    if value.is_empty() || !value.iter().all(u8::is_ascii_digit) {
        return Err(RetryAfterError::InvalidSyntax);
    }
    let mut parsed = 0_u16;
    for byte in value {
        parsed = parsed
            .checked_mul(10)
            .and_then(|current| current.checked_add(u16::from(*byte & 0x0f)))
            .ok_or(RetryAfterError::Overflow)?;
    }
    Ok(parsed)
}

fn month(value: &[u8]) -> Result<u8, RetryAfterError> {
    match value {
        b"Jan" => Ok(1),
        b"Feb" => Ok(2),
        b"Mar" => Ok(3),
        b"Apr" => Ok(4),
        b"May" => Ok(5),
        b"Jun" => Ok(6),
        b"Jul" => Ok(7),
        b"Aug" => Ok(8),
        b"Sep" => Ok(9),
        b"Oct" => Ok(10),
        b"Nov" => Ok(11),
        b"Dec" => Ok(12),
        _ => Err(RetryAfterError::InvalidSyntax),
    }
}

fn short_weekday(value: &[u8]) -> Result<u8, RetryAfterError> {
    match value {
        b"Sun" => Ok(0),
        b"Mon" => Ok(1),
        b"Tue" => Ok(2),
        b"Wed" => Ok(3),
        b"Thu" => Ok(4),
        b"Fri" => Ok(5),
        b"Sat" => Ok(6),
        _ => Err(RetryAfterError::InvalidSyntax),
    }
}

fn long_weekday(value: &[u8]) -> Result<u8, RetryAfterError> {
    match value {
        b"Sunday" => Ok(0),
        b"Monday" => Ok(1),
        b"Tuesday" => Ok(2),
        b"Wednesday" => Ok(3),
        b"Thursday" => Ok(4),
        b"Friday" => Ok(5),
        b"Saturday" => Ok(6),
        _ => Err(RetryAfterError::InvalidSyntax),
    }
}

fn is_leap_year(year: i64) -> bool {
    year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0)
}

fn days_in_month(year: i64, month: u8) -> u8 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn days_from_civil(year: i64, month: u8, day: u8) -> Result<i64, RetryAfterError> {
    let adjusted_year = year
        .checked_sub(if month <= 2 { 1 } else { 0 })
        .ok_or(RetryAfterError::Overflow)?;
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year.rem_euclid(400);
    let month = i64::from(month);
    let shifted_month = month
        .checked_add(if month > 2 { -3 } else { 9 })
        .ok_or(RetryAfterError::Overflow)?;
    let day_of_year = 153_i64
        .checked_mul(shifted_month)
        .and_then(|value| value.checked_add(2))
        .map(|value| value / 5)
        .and_then(|value| value.checked_add(i64::from(day)))
        .and_then(|value| value.checked_sub(1))
        .ok_or(RetryAfterError::Overflow)?;
    let day_of_era = year_of_era
        .checked_mul(365)
        .and_then(|value| value.checked_add(year_of_era / 4))
        .and_then(|value| value.checked_sub(year_of_era / 100))
        .and_then(|value| value.checked_add(day_of_year))
        .ok_or(RetryAfterError::Overflow)?;
    era.checked_mul(146_097)
        .and_then(|value| value.checked_add(day_of_era))
        .and_then(|value| value.checked_sub(719_468))
        .ok_or(RetryAfterError::Overflow)
}

struct CivilTime {
    year: i64,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

fn civil_time_at(now: WallClockTimestamp) -> Result<CivilTime, RetryAfterError> {
    let seconds = i64::try_from(now.get()).map_err(|_| RetryAfterError::Overflow)?;
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let shifted = days.checked_add(719_468).ok_or(RetryAfterError::Overflow)?;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era = day_of_era
        .checked_sub(day_of_era / 1_460)
        .and_then(|value| value.checked_add(day_of_era / 36_524))
        .and_then(|value| value.checked_sub(day_of_era / 146_096))
        .map(|value| value / 365)
        .ok_or(RetryAfterError::Overflow)?;
    let mut year = era
        .checked_mul(400)
        .and_then(|value| value.checked_add(year_of_era))
        .ok_or(RetryAfterError::Overflow)?;
    let elapsed = year_of_era
        .checked_mul(365)
        .and_then(|value| value.checked_add(year_of_era / 4))
        .and_then(|value| value.checked_sub(year_of_era / 100))
        .ok_or(RetryAfterError::Overflow)?;
    let day_of_year = day_of_era
        .checked_sub(elapsed)
        .ok_or(RetryAfterError::Overflow)?;
    let month_prime = day_of_year
        .checked_mul(5)
        .and_then(|value| value.checked_add(2))
        .map(|value| value / 153)
        .ok_or(RetryAfterError::Overflow)?;
    let month = month_prime
        .checked_add(if month_prime < 10 { 3 } else { -9 })
        .ok_or(RetryAfterError::Overflow)?;
    if month <= 2 {
        year = year.checked_add(1).ok_or(RetryAfterError::Overflow)?;
    }
    let day = day_of_year
        .checked_sub(
            153_i64
                .checked_mul(month_prime)
                .and_then(|value| value.checked_add(2))
                .map(|value| value / 5)
                .ok_or(RetryAfterError::Overflow)?,
        )
        .and_then(|value| value.checked_add(1))
        .ok_or(RetryAfterError::Overflow)?;
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    Ok(CivilTime {
        year,
        month: u8::try_from(month).map_err(|_| RetryAfterError::Overflow)?,
        day: u8::try_from(day).map_err(|_| RetryAfterError::Overflow)?,
        hour: u8::try_from(hour).map_err(|_| RetryAfterError::Overflow)?,
        minute: u8::try_from(minute).map_err(|_| RetryAfterError::Overflow)?,
        second: u8::try_from(second).map_err(|_| RetryAfterError::Overflow)?,
    })
}
