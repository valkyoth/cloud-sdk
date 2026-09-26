use super::*;
use cloud_sdk_sanitization::SecretBuffer;

struct Sink<'a> {
    expected: &'a [u8],
    committed: bool,
}
impl BlockingStreamSink for Sink<'_> {
    type Error = ();
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<usize, ()> {
        let n = bytes.len().min(3);
        if self.expected.get(..n) != bytes.get(..n) {
            return Err(());
        }
        self.expected = self.expected.get(n..).ok_or(())?;
        Ok(n)
    }
    fn commit(&mut self) -> Result<(), ()> {
        if !self.expected.is_empty() {
            return Err(());
        }
        self.committed = true;
        Ok(())
    }
    fn abort(&mut self, _: StreamPartialState) {
        self.committed = false;
    }
}
impl AsyncStreamSink for Sink<'_> {
    type Error = ();
    async fn write_chunk<'a>(&'a mut self, bytes: &'a [u8]) -> Result<usize, ()> {
        BlockingStreamSink::write_chunk(self, bytes)
    }
    async fn commit(&mut self) -> Result<(), ()> {
        BlockingStreamSink::commit(self)
    }
    fn abort(&mut self, state: StreamPartialState) {
        BlockingStreamSink::abort(self, state);
    }
}
fn verify<S>(
    f: &Fixture<'_>,
    request: TransportRequest<'_>,
    policy: RawResponsePolicy<'_>,
    upload: &AuthorizedUpload<'_, S>,
) {
    f.calls.fetch_add(1, Ordering::SeqCst);
    assert_eq!(upload.expected, f.endpoint_identity().fixture("origin"));
    assert_eq!(request.method(), Method::Put);
    assert_eq!(request.target().as_str(), "/api/v1/crates/new");
    assert!(request.body().is_empty());
    assert!(request.headers().get("authorization").is_none());
    assert!(request.headers().get("cookie").is_none());
    for (key, expected) in [
        ("accept", "application/json"),
        ("content-type", "application/octet-stream"),
        ("accept-encoding", "identity"),
    ] {
        assert_eq!(
            request
                .headers()
                .get(key)
                .fixture("header")
                .value()
                .as_str(),
            expected
        );
    }
    assert!(
        upload.authorization.as_str().as_bytes()
            == f.expected_auth.fixture("expected authorization")
    );
    assert_eq!(
        upload.policy.framing(),
        StreamFraming::Declared(u64::try_from(f.payload.len()).fixture("length"))
    );
    assert!(policy.admits_header("retry-after"));
}
fn finish(
    f: &Fixture<'_>,
    sink: &Sink<'_>,
    mut attempt: ResponseAttempt<'_, '_>,
) -> Result<(), ()> {
    assert!(sink.committed && sink.expected.is_empty());
    attempt
        .body_mut()
        .map_err(|_| ())?
        .get_mut(..f.wire.len())
        .ok_or(())?
        .copy_from_slice(f.wire);
    if f.media {
        attempt
            .headers_mut()
            .map_err(|_| ())?
            .try_push(
                "content-type",
                b"application/json",
                HeaderSensitivity::Public,
            )
            .map_err(|_| ())?;
    }
    if f.fail {
        return Err(());
    }
    attempt
        .commit(
            StatusCode::new(f.status).ok_or(())?,
            f.wire.len(),
            ResponseMetadata::EMPTY,
        )
        .map_err(|_| ())
}
impl BlockingRawUploadExecutor for Fixture<'_> {
    fn upload<S: BlockingStreamSource>(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        upload: AuthorizedUpload<'_, S>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), ()> {
        verify(self, request, policy, &upload);
        let mut scratch = SecretBuffer::new(upload.scratch);
        let attempt = response.begin_attempt().map_err(|_| ())?;
        let mut sink = Sink {
            expected: self.payload,
            committed: false,
        };
        drive_blocking_stream(
            upload.policy,
            upload.source,
            &mut sink,
            scratch.as_mut_slice(),
            &mut StreamOutcome::new(),
        )
        .map_err(|_| ())?;
        finish(self, &sink, attempt)
    }
}
impl AsyncRawUploadExecutor for Fixture<'_> {
    fn upload<'a, 'b: 'a, S: AsyncStreamSource + Send + 'a>(
        &'a self,
        request: TransportRequest<'a>,
        policy: RawResponsePolicy<'a>,
        upload: AuthorizedUpload<'a, S>,
        response: &'a mut ResponseWriter<'b>,
    ) -> impl Future<Output = Result<(), ()>> + Send + 'a {
        let mut scratch = SecretBuffer::new(upload.scratch);
        let attempt = response.begin_attempt();
        async move {
            let descriptor = AuthorizedUpload {
                expected: upload.expected,
                authorization: upload.authorization,
                source: upload.source,
                policy: upload.policy,
                scratch: scratch.as_mut_slice(),
            };
            verify(self, request, policy, &descriptor);
            let attempt = attempt.map_err(|_| ())?;
            let mut sink = Sink {
                expected: self.payload,
                committed: false,
            };
            drive_async_stream(
                descriptor.policy,
                descriptor.source,
                &mut sink,
                descriptor.scratch,
                &mut StreamOutcome::new(),
            )
            .await
            .map_err(|_| ())?;
            if self.pending {
                core::future::pending::<()>().await;
            }
            finish(self, &sink, attempt)
        }
    }
}
impl LocalRawUploadExecutor for Local<'_> {
    fn upload_local<'a, 'b: 'a, S: LocalAsyncStreamSource + 'a>(
        &'a self,
        request: TransportRequest<'a>,
        policy: RawResponsePolicy<'a>,
        upload: AuthorizedUpload<'a, S>,
        response: &'a mut ResponseWriter<'b>,
    ) -> impl Future<Output = Result<(), ()>> + 'a {
        let mut scratch = SecretBuffer::new(upload.scratch);
        let attempt = response.begin_attempt();
        async move {
            let descriptor = AuthorizedUpload {
                expected: upload.expected,
                authorization: upload.authorization,
                source: upload.source,
                policy: upload.policy,
                scratch: scratch.as_mut_slice(),
            };
            verify(&self.0, request, policy, &descriptor);
            let attempt = attempt.map_err(|_| ())?;
            let mut sink = Sink {
                expected: self.0.payload,
                committed: false,
            };
            drive_local_stream(
                descriptor.policy,
                descriptor.source,
                &mut sink,
                descriptor.scratch,
                &mut StreamOutcome::new(),
            )
            .await
            .map_err(|_| ())?;
            if self.0.pending {
                core::future::pending::<()>().await;
            }
            finish(&self.0, &sink, attempt)
        }
    }
}
