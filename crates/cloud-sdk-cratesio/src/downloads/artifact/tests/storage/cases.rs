use super::*;

#[test]
fn real_storage_commits_only_verified_bytes_in_all_modes() {
    for mode in 0..3 {
        let directory = Directory::new();
        let mut sink = FileSink::new(&directory);
        sink.not_published();
        let mut scratch = [0xa5; 2];
        execute(mode, &transport(), &mut sink, &mut scratch).fixture("verified download");
        assert!(sink.committed);
        assert_eq!(sink.aborts, 0);
        assert_eq!(fs::read(&sink.published).fixture("published bytes"), b"abc");
        assert_eq!(scratch, [0; 2]);
        let published = sink.published.clone();
        let temporary = sink.temporary.clone();
        drop(sink);
        assert_eq!(
            fs::symlink_metadata(temporary)
                .err()
                .fixture("temporary removed")
                .kind(),
            io::ErrorKind::NotFound
        );
        assert_eq!(fs::read(published).fixture("durable visible bytes"), b"abc");
    }
}

#[test]
fn real_storage_rolls_back_checksum_framing_io_and_commit_errors() {
    for mode in 0..3 {
        for case in 0..7 {
            let directory = Directory::new();
            let mut sink = FileSink::new(&directory);
            let mut transport = transport();
            match case {
                0 => transport.bytes = b"abd",
                1 => transport.bytes = b"ab",
                2 => transport.bytes = b"abcd",
                3 => transport.length = Some(4),
                4 => transport.fail = true,
                5 => sink.fail_write = true,
                _ => sink.fail_commit = true,
            }
            let mut scratch = [0xa5; 2];
            assert!(execute(mode, &transport, &mut sink, &mut scratch).is_err());
            sink.rolled_back();
            sink.not_published();
            assert_eq!(scratch, [0; 2]);
        }
    }
}

#[test]
fn publication_collision_never_replaces_existing_files_or_symlinks() {
    for mode in 0..3 {
        for symlink in [false, true] {
            let directory = Directory::new();
            let mut sink = FileSink::new(&directory);
            let original = directory.0.join("original");
            fs::write(&original, b"keep original").fixture("original fixture");
            if symlink {
                test_std::os::unix::fs::symlink(&original, &sink.published)
                    .fixture("symlink fixture");
            } else {
                fs::write(&sink.published, b"keep original").fixture("existing destination");
            }
            let mut scratch = [0xa5; 2];
            assert!(execute(mode, &transport(), &mut sink, &mut scratch).is_err());
            sink.rolled_back();
            assert_eq!(
                fs::read(&original).fixture("original unchanged"),
                b"keep original"
            );
            assert_eq!(
                fs::read(&sink.published).fixture("destination unchanged"),
                b"keep original"
            );
            assert_eq!(
                fs::symlink_metadata(&sink.published)
                    .fixture("destination metadata")
                    .file_type()
                    .is_symlink(),
                symlink
            );
            assert_eq!(scratch, [0; 2]);
        }
    }
}

fn cancel<F: Future>(future: F, case: u8, temporary: &PathBuf, published: &PathBuf) {
    let mut future = core::pin::pin!(future);
    if case != 0 {
        assert!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
    }
    let expected: &[u8] = match case {
        0 => b"",
        1 => b"ab",
        2 => b"a",
        _ => b"abc",
    };
    assert_eq!(
        fs::read(temporary).fixture("tentative bytes before cancellation"),
        expected
    );
    assert_eq!(
        fs::symlink_metadata(published)
            .err()
            .fixture("not published before cancellation")
            .kind(),
        io::ErrorKind::NotFound
    );
}

#[test]
fn real_storage_cancellation_removes_tentative_bytes_at_each_boundary() {
    for local in [false, true] {
        for case in 0..4 {
            let directory = Directory::new();
            let mut sink = FileSink::new(&directory);
            let mut transport = transport();
            match case {
                1 => transport.pending = true,
                2 => sink.pause = Pause::Write,
                3 => sink.pause = Pause::Commit,
                _ => {}
            }
            let mut scratch = [0xa5; 2];
            let checksum = crate::downloads::Sha256Checksum::new();
            let temporary = sink.temporary.clone();
            let published = sink.published.clone();
            if local {
                cancel(
                    download().execute_local(
                        &transport,
                        identity(),
                        &mut sink,
                        checksum,
                        &mut scratch,
                    ),
                    case,
                    &temporary,
                    &published,
                );
            } else {
                cancel(
                    send(download().execute_async(
                        &transport,
                        identity(),
                        &mut sink,
                        checksum,
                        &mut scratch,
                    )),
                    case,
                    &temporary,
                    &published,
                );
            }
            sink.rolled_back();
            sink.not_published();
            assert_eq!(scratch, [0; 2]);
        }
    }
}
