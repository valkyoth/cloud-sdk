use super::*;

#[test]
fn strict_publish_response_does_not_silently_claim_minimal_cargo_compatibility() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let bytes = bytes();
    let token = token::<Api>(CredentialOrigin::Production, &bytes);
    let expected = framed();
    for wire in [
        b"{}".as_slice(),
        br#"{"warnings":{"invalid_categories":[],"invalid_badges":[],"other":[]}}"#,
    ] {
        for mode in 0..3 {
            reset_test_gate();
            let mut fixture = Fixture::new(Method::Put, "/api/v1/crates/new", &expected, wire, 200);
            fixture.expected_auth = Some(&bytes);
            let local = Local(fixture, Rc::new(()));
            let client = RegistryClient::production(&local.0, identity(), 16384).fixture("client");
            let local_client =
                RegistryClient::production(&local, identity(), 16384).fixture("local");
            let mut buffers = Buffers::new();
            let mut source = SlicePackage::new(b"crate");
            let permit = request().confirm_api(&token);
            let result = match mode {
                0 => client.publish(permit, &mut source, buffers.parts()),
                1 => complete(local_client.publish_local(permit, &mut source, buffers.parts())),
                2 => complete(send(client.publish_async(
                    permit,
                    &mut source,
                    buffers.parts(),
                ))),
                _ => unreachable!("mode"),
            };
            assert!(matches!(result, Err(DiscoveryExecutionError::Model(_))));
            assert_eq!(local.0.calls.load(Ordering::SeqCst), 1);
            buffers.cleared();
        }
    }
    reset_test_gate();
}

#[test]
fn publication_failures_do_not_retry_and_clear_all_regions() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let bytes = bytes();
    let expected = framed();
    for mode in 0..3 {
        for fault in 0..9 {
            reset_test_gate();
            let mut token = token::<Api>(
                if fault == 0 {
                    CredentialOrigin::Staging
                } else {
                    CredentialOrigin::Production
                },
                &bytes,
            );
            if fault == 1 {
                token.clear();
            }
            let mut fixture =
                Fixture::new(Method::Put, "/api/v1/crates/new", &expected, RESPONSE, 200);
            fixture.expected_auth = Some(&bytes);
            match fault {
                4 => fixture.wire = b"{}",
                5 => fixture.status = 403,
                6 => fixture.media = false,
                7 => fixture.fail = true,
                8 => fixture.wire = br#"{"errors":[{"detail":"rejected"}]}"#,
                _ => {}
            }
            let local = Local(fixture, Rc::new(()));
            let client = RegistryClient::production(&local.0, identity(), 16384).fixture("client");
            let local_client =
                RegistryClient::production(&local, identity(), 16384).fixture("local");
            let mut buffers = Buffers::new();
            let data = match fault {
                2 => b"crat".as_slice(),
                3 => b"crates",
                _ => b"crate",
            };
            let mut source = SlicePackage::new(data);
            let mut local_source = LocalSource(SlicePackage::new(data), Rc::new(()));
            let permit = request().confirm_api(&token);
            let result = match mode {
                0 => client.publish(permit, &mut local_source, buffers.parts()),
                1 => {
                    complete(local_client.publish_local(permit, &mut local_source, buffers.parts()))
                }
                2 => complete(send(client.publish_async(
                    permit,
                    &mut source,
                    buffers.parts(),
                ))),
                _ => unreachable!("mode"),
            };
            assert!(result.is_err());
            buffers.cleared();
            assert_eq!(
                local.0.calls.load(Ordering::SeqCst),
                usize::from(fault >= 2)
            );
            if fault < 2 {
                drop(
                    crate::wire::OfficialApiGate::new(identity())
                        .begin()
                        .fixture("unused admission"),
                );
                let mut out = [0; 5];
                let read = if mode == 2 {
                    BlockingStreamSource::read_chunk(&mut source, &mut out)
                } else {
                    BlockingStreamSource::read_chunk(&mut local_source, &mut out)
                };
                assert_eq!(read, Ok(StreamRead::Chunk(5)));
                assert_eq!(&out, b"crate");
            }
        }
    }
    reset_test_gate();
}

#[test]
fn publication_future_cleanup_is_armed_before_poll_and_during_exchange() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let bytes = bytes();
    let token = token::<Api>(CredentialOrigin::Production, &bytes);
    let expected = framed();
    for local_mode in [false, true] {
        for polled in [false, true] {
            reset_test_gate();
            let mut fixture =
                Fixture::new(Method::Put, "/api/v1/crates/new", &expected, RESPONSE, 200);
            fixture.expected_auth = Some(&bytes);
            fixture.pending = true;
            let local = Local(fixture, Rc::new(()));
            let client = RegistryClient::production(&local.0, identity(), 16384).fixture("client");
            let local_client =
                RegistryClient::production(&local, identity(), 16384).fixture("local");
            let mut buffers = Buffers::new();
            let mut source = SlicePackage::new(b"crate");
            let mut local_source = LocalSource(SlicePackage::new(b"crate"), Rc::new(()));
            let permit = request().confirm_api(&token);
            if local_mode {
                let mut future = core::pin::pin!(local_client.publish_local(
                    permit,
                    &mut local_source,
                    buffers.parts()
                ));
                if polled {
                    assert!(
                        future
                            .as_mut()
                            .poll(&mut Context::from_waker(Waker::noop()))
                            .is_pending()
                    );
                }
            } else {
                let mut future = core::pin::pin!(send(client.publish_async(
                    permit,
                    &mut source,
                    buffers.parts()
                )));
                if polled {
                    assert!(
                        future
                            .as_mut()
                            .poll(&mut Context::from_waker(Waker::noop()))
                            .is_pending()
                    );
                }
            }
            buffers.cleared();
            assert_eq!(local.0.calls.load(Ordering::SeqCst), usize::from(polled));
        }
    }
    reset_test_gate();
}
