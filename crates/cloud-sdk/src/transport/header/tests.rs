use super::{
    ContentType, HeaderError, HeaderSensitivity, HeaderValue, MAX_HEADER_NAME_BYTES,
    MAX_HEADER_VALUE_BYTES, MAX_REQUEST_HEADER_BYTES, MAX_REQUEST_HEADERS,
    MAX_RESPONSE_HEADER_BYTES, MAX_RESPONSE_HEADERS, MediaType, RequestHeader, RequestHeaders,
    ResponseHeaders,
};
use core::fmt::Write;

#[test]
fn validates_names_values_and_typed_common_headers() {
    assert!(RequestHeader::new("x-request-id", "abc 123").is_ok());
    assert_eq!(RequestHeader::new("", "value"), Err(HeaderError::EmptyName));
    assert_eq!(
        RequestHeader::new("bad name", "value"),
        Err(HeaderError::InvalidName)
    );
    assert_eq!(
        RequestHeader::new("x-test", "bad\r\ninjected: true"),
        Err(HeaderError::InvalidValue)
    );
    assert_eq!(
        RequestHeader::new("x-test", " padded"),
        Err(HeaderError::InvalidValue)
    );
    assert_eq!(
        RequestHeader::new("content-type", "not-a-media-type"),
        Err(HeaderError::InvalidContentType)
    );

    let accept = RequestHeader::accept(MediaType::JSON);
    let content_type = RequestHeader::content_type(ContentType::JSON);
    assert_eq!(accept.name().as_str(), "accept");
    assert_eq!(accept.value().as_str(), "application/json");
    assert_eq!(content_type.name().as_str(), "content-type");
    assert_eq!(
        RequestHeader::new("X-Trace", "one").map(RequestHeader::name),
        RequestHeader::new("x-trace", "two").map(RequestHeader::name)
    );
}

#[test]
fn rejects_every_reserved_authority_framing_and_auth_name() {
    for name in [
        "Host",
        "content-length",
        "Transfer-Encoding",
        "connection",
        "keep-alive",
        "TE",
        "trailer",
        "upgrade",
        "proxy-authenticate",
        "Proxy-Authorization",
        "proxy-connection",
        "Authorization",
    ] {
        assert_eq!(
            RequestHeader::new(name, "value"),
            Err(HeaderError::ReservedRequestHeader),
            "{name}"
        );
    }
}

#[test]
fn duplicate_names_fail_regardless_of_case_or_value() {
    let first = RequestHeader::new("X-Trace", "same");
    let identical = RequestHeader::new("x-trace", "same");
    let conflicting = RequestHeader::new("x-TRACE", "different");
    let (Ok(first), Ok(identical), Ok(conflicting)) = (first, identical, conflicting) else {
        return;
    };
    assert_eq!(
        RequestHeaders::new(&[first, identical]),
        Err(HeaderError::DuplicateName)
    );
    assert_eq!(
        RequestHeaders::new(&[first, conflicting]),
        Err(HeaderError::DuplicateName)
    );
}

#[test]
fn request_encoding_is_exact_and_failure_is_atomic() {
    let first = RequestHeader::new("x-one", "alpha");
    let second = RequestHeader::sensitive("x-secret", "token");
    if let (Ok(first), Ok(second)) = (first, second) {
        let entries = [first, second];
        let headers = RequestHeaders::new(&entries);
        assert!(headers.is_ok());
        if let Ok(headers) = headers {
            let mut output = [0xa5; 64];
            let len = headers.encode_http1(&mut output);
            assert_eq!(len, Ok(headers.encoded_len()));
            assert_eq!(
                output.get(..headers.encoded_len()),
                Some(b"x-one: alpha\r\nx-secret: token\r\n".as_slice())
            );

            let mut short = [0xa5; 8];
            assert_eq!(
                headers.encode_http1(&mut short),
                Err(HeaderError::OutputTooSmall)
            );
            assert_eq!(short, [0xa5; 8]);
        }
    }
}

#[test]
fn all_request_capacity_boundaries_are_enforced() {
    let long_name = "x".repeat(MAX_HEADER_NAME_BYTES + 1);
    let long_value = "x".repeat(MAX_HEADER_VALUE_BYTES + 1);
    assert_eq!(
        RequestHeader::new(&long_name, "value"),
        Err(HeaderError::NameTooLong)
    );
    assert_eq!(
        RequestHeader::new("x-test", &long_value),
        Err(HeaderError::ValueTooLong)
    );

    let entry = RequestHeader::new("x", "").ok();
    if let Some(entry) = entry {
        let entries = [entry; MAX_REQUEST_HEADERS + 1];
        assert_eq!(
            RequestHeaders::new(&entries),
            Err(HeaderError::TooManyHeaders)
        );
    }

    let value = "x".repeat(MAX_HEADER_VALUE_BYTES);
    let mut entries = [RequestHeader::accept(MediaType::JSON); MAX_REQUEST_HEADERS];
    for (index, entry) in entries.iter_mut().enumerate() {
        let name = match index {
            0 => "x-00",
            1 => "x-01",
            2 => "x-02",
            3 => "x-03",
            4 => "x-04",
            5 => "x-05",
            6 => "x-06",
            7 => "x-07",
            8 => "x-08",
            9 => "x-09",
            10 => "x-10",
            11 => "x-11",
            12 => "x-12",
            13 => "x-13",
            14 => "x-14",
            15 => "x-15",
            16 => "x-16",
            17 => "x-17",
            18 => "x-18",
            19 => "x-19",
            20 => "x-20",
            21 => "x-21",
            22 => "x-22",
            23 => "x-23",
            24 => "x-24",
            25 => "x-25",
            26 => "x-26",
            27 => "x-27",
            28 => "x-28",
            29 => "x-29",
            30 => "x-30",
            _ => "x-31",
        };
        if let Ok(value) = RequestHeader::new(name, &value) {
            *entry = value;
        }
    }
    assert_eq!(
        RequestHeaders::new(&entries),
        Err(HeaderError::AggregateTooLarge)
    );
}

#[test]
fn response_headers_are_owned_bounded_ordered_and_redacted() {
    let mut headers = ResponseHeaders::new();
    assert_eq!(
        headers.try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public
        ),
        Ok(())
    );
    assert_eq!(
        headers.try_push("set-cookie", b"secret=1", HeaderSensitivity::Sensitive),
        Ok(())
    );
    assert_eq!(headers.len(), 2);
    assert_eq!(
        headers.get("Content-Type").map(|header| header.value()),
        Some(b"application/json".as_slice())
    );
    let mut debug = DebugBuffer::new();
    assert!(write!(&mut debug, "{headers:?}").is_ok());
    assert!(debug.as_str().contains("[redacted]"));
    assert!(!debug.as_str().contains("secret=1"));
    debug.clear();
    assert!(write!(&mut debug, "{:?}", headers.get("set-cookie")).is_ok());
    assert!(!debug.as_str().contains("secret=1"));

    let snapshot = headers;
    assert_eq!(
        headers.try_push("CONTENT-TYPE", b"text/plain", HeaderSensitivity::Public),
        Err(HeaderError::DuplicateName)
    );
    assert_eq!(headers, snapshot);
    assert_eq!(
        headers.try_push("x-bad", b"ok\r\nbad", HeaderSensitivity::Public),
        Err(HeaderError::InvalidValue)
    );
    assert_eq!(headers, snapshot);
}

#[test]
fn response_count_and_aggregate_limits_fail_atomically() {
    let mut count = ResponseHeaders::new();
    for index in 0..MAX_RESPONSE_HEADERS {
        let names = [
            "x-00", "x-01", "x-02", "x-03", "x-04", "x-05", "x-06", "x-07", "x-08", "x-09", "x-10",
            "x-11", "x-12", "x-13", "x-14", "x-15", "x-16", "x-17", "x-18", "x-19", "x-20", "x-21",
            "x-22", "x-23", "x-24", "x-25", "x-26", "x-27", "x-28", "x-29", "x-30", "x-31",
        ];
        let name = names.get(index).copied().unwrap_or_default();
        assert_eq!(count.try_push(name, b"", HeaderSensitivity::Public), Ok(()));
    }
    let snapshot = count;
    assert_eq!(
        count.try_push("x-over", b"", HeaderSensitivity::Public),
        Err(HeaderError::TooManyHeaders)
    );
    assert_eq!(count, snapshot);

    let mut aggregate = ResponseHeaders::new();
    let value = [b'x'; MAX_HEADER_VALUE_BYTES];
    for index in 0..8 {
        let names = ["x-0", "x-1", "x-2", "x-3", "x-4", "x-5", "x-6", "x-7"];
        let name = names.get(index).copied().unwrap_or_default();
        let result = aggregate.try_push(name, &value, HeaderSensitivity::Public);
        if index < 7 {
            assert_eq!(result, Ok(()));
        }
    }
    let snapshot = aggregate;
    assert_eq!(
        aggregate.try_push("x-over", &value, HeaderSensitivity::Public),
        Err(HeaderError::AggregateTooLarge)
    );
    assert_eq!(aggregate, snapshot);
}

#[test]
fn exact_name_and_value_bounds_are_admitted() {
    let name = "x".repeat(MAX_HEADER_NAME_BYTES);
    let value = "v".repeat(MAX_HEADER_VALUE_BYTES);
    assert!(RequestHeader::new(&name, &value).is_ok());
    assert!(HeaderValue::new("").is_ok());

    let mut response = ResponseHeaders::new();
    assert_eq!(
        response.try_push(&name, value.as_bytes(), HeaderSensitivity::Public),
        Ok(())
    );
    let long_name = "x".repeat(MAX_HEADER_NAME_BYTES + 1);
    let long_value = "v".repeat(MAX_HEADER_VALUE_BYTES + 1);
    assert_eq!(
        response.try_push(&long_name, b"", HeaderSensitivity::Public),
        Err(HeaderError::NameTooLong)
    );
    assert_eq!(
        response.try_push(
            "x-too-long",
            long_value.as_bytes(),
            HeaderSensitivity::Public
        ),
        Err(HeaderError::ValueTooLong)
    );
}

#[test]
fn exact_aggregate_request_and_response_boundaries_are_admitted() {
    let full = "x".repeat(MAX_HEADER_VALUE_BYTES);
    let names = ["x-0", "x-1", "x-2", "x-3", "x-4", "x-5", "x-6"];
    let (Ok(first_0), Ok(first_1), Ok(first_2), Ok(first_3), Ok(first_4), Ok(first_5), Ok(first_6)) = (
        RequestHeader::new("x-0", &full),
        RequestHeader::new("x-1", &full),
        RequestHeader::new("x-2", &full),
        RequestHeader::new("x-3", &full),
        RequestHeader::new("x-4", &full),
        RequestHeader::new("x-5", &full),
        RequestHeader::new("x-6", &full),
    ) else {
        return;
    };
    let first = [
        first_0, first_1, first_2, first_3, first_4, first_5, first_6,
    ];
    let first_headers = RequestHeaders::new(&first);
    assert!(first_headers.is_ok());
    let Ok(first_headers) = first_headers else {
        return;
    };
    let final_name = "x-final";
    let overhead = final_name.len().checked_add(4).unwrap_or_default();
    let final_len = MAX_REQUEST_HEADER_BYTES
        .checked_sub(first_headers.encoded_len())
        .and_then(|remaining| remaining.checked_sub(overhead))
        .unwrap_or_default();
    let final_value = "y".repeat(final_len);
    let final_header = RequestHeader::new(final_name, &final_value);
    assert!(final_header.is_ok());
    let Ok(final_header) = final_header else {
        return;
    };
    let entries = [
        first_0,
        first_1,
        first_2,
        first_3,
        first_4,
        first_5,
        first_6,
        final_header,
    ];
    let exact = RequestHeaders::new(&entries);
    assert!(exact.is_ok());
    assert_eq!(
        exact.map(RequestHeaders::encoded_len),
        Ok(MAX_REQUEST_HEADER_BYTES)
    );

    let mut response = ResponseHeaders::new();
    for name in names {
        assert_eq!(
            response.try_push(name, full.as_bytes(), HeaderSensitivity::Public),
            Ok(())
        );
    }
    let response_final_len = MAX_RESPONSE_HEADER_BYTES
        .checked_sub(response.encoded_len())
        .and_then(|remaining| remaining.checked_sub(overhead))
        .unwrap_or_default();
    let response_final = "z".repeat(response_final_len);
    assert_eq!(
        response.try_push(
            final_name,
            response_final.as_bytes(),
            HeaderSensitivity::Public
        ),
        Ok(())
    );
    assert_eq!(response.encoded_len(), MAX_RESPONSE_HEADER_BYTES);
    let snapshot = response;
    assert_eq!(
        response.try_push("x-over", b"", HeaderSensitivity::Public),
        Err(HeaderError::AggregateTooLarge)
    );
    assert_eq!(response, snapshot);
}

struct DebugBuffer {
    bytes: [u8; 256],
    len: usize,
}

impl DebugBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 256],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
    }

    fn clear(&mut self) {
        self.bytes.fill(0);
        self.len = 0;
    }
}

impl Write for DebugBuffer {
    fn write_str(&mut self, value: &str) -> core::fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(core::fmt::Error)?;
        let output = self.bytes.get_mut(self.len..end).ok_or(core::fmt::Error)?;
        output.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}
