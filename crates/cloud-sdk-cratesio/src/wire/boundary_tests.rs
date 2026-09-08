use super::response_tests::fixture;
use super::{CratesIoWireError, JsonResponsePolicy, ProviderErrorKind};
use alloc::{format, vec};
use cloud_sdk::incremental_json::{
    IncrementalJsonError, IncrementalJsonEvent, IncrementalJsonProgress, IncrementalJsonVisitor,
    VisitControl,
};
use cloud_sdk::rate_limit::WallClockTimestamp;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

#[test]
fn error_entry_limit_is_exact_and_unannotated_error_statuses_are_classified() {
    for count in [256_usize, 257] {
        let input = format!(
            "{{\"errors\":[{}]}}",
            alloc::vec!["{\"detail\":\"\"}"; count].join(",")
        );
        let mut body = vec![0; input.len()];
        let mut headers = [0; 256];
        let response = fixture(
            &mut body,
            &mut headers,
            input.as_bytes(),
            200,
            Some(b"application/json"),
            None,
        );
        let error = JsonResponsePolicy::new(StatusCode::OK, input.len())
            .unwrap_or_else(|_| unreachable!("policy fixture"))
            .admit(response, WallClockTimestamp::new(0))
            .err()
            .unwrap_or_else(|| unreachable!("errors must never be success"));
        match (count, error) {
            (256, CratesIoWireError::Provider(provider)) => assert_eq!(provider.count(), 256),
            (257, CratesIoWireError::Envelope) => {}
            _ => unreachable!("incorrect error-count boundary"),
        }
    }
    for code in [429, 503] {
        let mut body = [0; 8];
        let mut headers = [0; 256];
        let response = fixture(
            &mut body,
            &mut headers,
            b"{}",
            code,
            Some(b"application/json"),
            None,
        );
        let result = JsonResponsePolicy::new(StatusCode::OK, 8)
            .unwrap_or_else(|_| unreachable!("policy fixture"))
            .admit(response, WallClockTimestamp::new(0));
        let Err(CratesIoWireError::Provider(error)) = result else {
            unreachable!("HTTP error not classified");
        };
        assert_eq!(error.retry_after(), None);
        assert_eq!(error.count(), 0);
        assert_eq!(
            error.kind(),
            if code == 429 {
                ProviderErrorKind::RateLimited
            } else {
                ProviderErrorKind::Unavailable
            }
        );
    }
}

#[test]
fn encoded_bodies_and_duplicate_headers_cannot_bypass_admission() {
    for encoding in [b"identity".as_slice(), b"gzip", b"br", b"identity,gzip"] {
        let mut body = [0; 16];
        let mut headers = [0; 256];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut headers);
        let mut attempt = response
            .writer()
            .begin_attempt()
            .unwrap_or_else(|_| unreachable!("attempt fixture"));
        attempt
            .body_mut()
            .unwrap_or_else(|_| unreachable!("body fixture"))
            .get_mut(..2)
            .unwrap_or_else(|| unreachable!("body prefix fixture"))
            .copy_from_slice(b"{}");
        let output = attempt
            .headers_mut()
            .unwrap_or_else(|_| unreachable!("headers fixture"));
        assert!(
            output
                .try_push(
                    "content-type",
                    b"application/json",
                    HeaderSensitivity::Public
                )
                .is_ok()
        );
        assert!(
            output
                .try_push("CONTENT-TYPE", b"text/html", HeaderSensitivity::Public)
                .is_err()
        );
        assert!(
            output
                .try_push("retry-after", b"1", HeaderSensitivity::Public)
                .is_ok()
        );
        assert!(
            output
                .try_push("Retry-After", b"0", HeaderSensitivity::Public)
                .is_err()
        );
        assert!(
            output
                .try_push("content-encoding", encoding, HeaderSensitivity::Public)
                .is_ok()
        );
        assert!(
            attempt
                .commit(StatusCode::OK, 2, ResponseMetadata::EMPTY)
                .is_ok()
        );
        drop(attempt);
        let result = JsonResponsePolicy::new(StatusCode::OK, 16)
            .unwrap_or_else(|_| unreachable!("policy fixture"))
            .admit(response, WallClockTimestamp::new(0));
        assert_eq!(result.is_ok(), encoding == b"identity");
    }
}

#[test]
fn visitor_failure_and_early_stop_clear_admitted_storage() {
    struct Visitor(bool);
    impl IncrementalJsonVisitor for Visitor {
        type Error = &'static str;
        fn visit(&mut self, _: IncrementalJsonEvent<'_>) -> Result<VisitControl, Self::Error> {
            if self.0 {
                Err("sensitive visitor error")
            } else {
                Ok(VisitControl::Stop)
            }
        }
    }
    for fails in [true, false] {
        let mut body = [0xa5; 32];
        let mut headers = [0xa5; 256];
        let response = fixture(
            &mut body,
            &mut headers,
            b"{}",
            200,
            Some(b"application/json"),
            None,
        );
        let success = JsonResponsePolicy::new(StatusCode::OK, 32)
            .unwrap_or_else(|_| unreachable!("policy fixture"))
            .admit(response, WallClockTimestamp::new(0))
            .unwrap_or_else(|_| unreachable!("valid response fixture"));
        let result = success.visit(&mut Visitor(fails));
        if fails {
            assert!(matches!(result, Err(IncrementalJsonError::Visitor(_))));
            assert!(!format!("{result:?}").contains("sensitive visitor error"));
        } else {
            assert_eq!(result, Ok(IncrementalJsonProgress::Stopped));
        }
        assert!(body.iter().all(|byte| *byte == 0));
        assert!(headers.iter().all(|byte| *byte == 0));
    }
}
