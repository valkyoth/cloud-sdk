use crate::rate_limit::{HttpDate, RetryAfter, RetryAfterError, WallClockTimestamp};

const NOW_2026: WallClockTimestamp = WallClockTimestamp::new(1_767_225_600);

#[test]
fn accepts_all_required_http_date_forms() {
    for value in [
        b"Sun, 06 Nov 1994 08:49:37 GMT".as_slice(),
        b"Sunday, 06-Nov-94 08:49:37 GMT".as_slice(),
        b"Sun Nov  6 08:49:37 1994".as_slice(),
    ] {
        assert_eq!(
            RetryAfter::parse(value, NOW_2026),
            Ok(RetryAfter::HttpDate(HttpDate::new(784_111_777)))
        );
    }
}

#[test]
fn accepts_delay_zero_boundary_and_leap_second() {
    assert_eq!(
        RetryAfter::parse(b"0", NOW_2026),
        Ok(RetryAfter::Delay(crate::rate_limit::DelaySeconds::new(0)))
    );
    assert_eq!(
        RetryAfter::parse(b"Sat, 31 Dec 2016 23:59:60 GMT", NOW_2026),
        Ok(RetryAfter::HttpDate(HttpDate::new(1_483_228_800)))
    );
    assert_eq!(
        RetryAfter::parse(b"18446744073709551615", NOW_2026),
        Ok(RetryAfter::Delay(crate::rate_limit::DelaySeconds::new(
            u64::MAX,
        )))
    );
}

#[test]
fn rejects_overflow_invalid_dates_and_wrong_weekdays() {
    assert_eq!(
        RetryAfter::parse(b"", NOW_2026),
        Err(RetryAfterError::Empty)
    );
    assert_eq!(
        RetryAfter::parse(b"18446744073709551616", NOW_2026),
        Err(RetryAfterError::Overflow)
    );
    assert_eq!(
        RetryAfter::parse(b"Sun, 31 Feb 2026 00:00:00 GMT", NOW_2026),
        Err(RetryAfterError::InvalidDate)
    );
    assert_eq!(
        RetryAfter::parse(b"Mon, 06 Nov 1994 08:49:37 GMT", NOW_2026),
        Err(RetryAfterError::WeekdayMismatch)
    );
}

#[test]
fn rfc850_year_uses_the_required_fifty_year_window() {
    assert_eq!(
        RetryAfter::parse(b"Sunday, 06-Nov-94 08:49:37 GMT", NOW_2026),
        Ok(RetryAfter::HttpDate(HttpDate::new(784_111_777)))
    );
}

#[test]
fn rfc850_year_uses_the_complete_fifty_year_timestamp_boundary() {
    assert_eq!(
        RetryAfter::parse(b"Tuesday, 31-Dec-75 23:59:59 GMT", NOW_2026),
        Ok(RetryAfter::HttpDate(HttpDate::new(3_345_062_399)))
    );
    assert_eq!(
        RetryAfter::parse(b"Wednesday, 01-Jan-76 00:00:00 GMT", NOW_2026),
        Ok(RetryAfter::HttpDate(HttpDate::new(3_345_062_400)))
    );
    assert_eq!(
        RetryAfter::parse(b"Thursday, 01-Jan-76 00:00:01 GMT", NOW_2026),
        Ok(RetryAfter::HttpDate(HttpDate::new(189_302_401)))
    );
    assert_eq!(
        RetryAfter::parse(b"Wednesday, 01-Jan-76 00:00:01 GMT", NOW_2026),
        Err(RetryAfterError::WeekdayMismatch)
    );
}
