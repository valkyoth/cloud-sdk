//! Real host-filesystem qualification, not a public or asynchronous file adapter.
use super::*;
use crate::std as test_std;
use test_std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

mod cases;

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = test_std::env::temp_dir().join(alloc::format!(
            "cloud-sdk-artifact-{}-{id}",
            test_std::process::id()
        ));
        // No reuse: a pre-existing file, directory or symlink fails the fixture.
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .fixture("create private storage fixture");
        assert_eq!(
            fs::metadata(&path)
                .fixture("directory metadata")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).fixture("remove private storage fixture");
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pause {
    Never,
    Write,
    Commit,
}

struct FileSink {
    file: Option<File>,
    temporary: PathBuf,
    published: PathBuf,
    aborts: usize,
    committed: bool,
    fail_write: bool,
    fail_commit: bool,
    pause: Pause,
}
impl FileSink {
    fn new(directory: &Directory) -> Self {
        let temporary = directory.0.join("tentative");
        let published = directory.0.join("download.crate");
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&temporary)
            .fixture("create tentative file");
        Self {
            file: Some(file),
            temporary,
            published,
            aborts: 0,
            committed: false,
            fail_write: false,
            fail_commit: false,
            pause: Pause::Never,
        }
    }
    fn rolled_back(&self) {
        assert!(!self.committed);
        assert_eq!(self.aborts, 1);
        assert!(self.file.is_none());
        assert_eq!(
            fs::symlink_metadata(&self.temporary)
                .err()
                .fixture("tentative removed")
                .kind(),
            io::ErrorKind::NotFound
        );
    }
    fn not_published(&self) {
        assert_eq!(
            fs::symlink_metadata(&self.published)
                .err()
                .fixture("no final artifact")
                .kind(),
            io::ErrorKind::NotFound
        );
    }
}
impl BlockingStreamSink for FileSink {
    type Error = io::Error;
    fn write_chunk(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.fail_write {
            return Err(io::Error::other("injected file write failure"));
        }
        let first = bytes
            .get(..1)
            .ok_or_else(|| io::Error::other("empty write"))?;
        self.file
            .as_mut()
            .ok_or_else(|| io::Error::other("closed sink"))?
            .write(first)
    }
    fn commit(&mut self) -> io::Result<()> {
        if self.fail_commit {
            return Err(io::Error::other("injected commit failure"));
        }
        self.file
            .as_mut()
            .ok_or_else(|| io::Error::other("closed sink"))?
            .sync_all()?;
        self.file.take();
        // One non-overwriting publication on the same filesystem. No fallible
        // operation follows publication; temporary-name cleanup occurs on drop.
        fs::hard_link(&self.temporary, &self.published)?;
        self.committed = true;
        Ok(())
    }
    fn abort(&mut self, _: StreamPartialState) {
        self.file.take();
        fs::remove_file(&self.temporary).fixture("rollback tentative file");
        self.aborts = self.aborts.checked_add(1).fixture("abort count");
    }
}
impl AsyncStreamSink for FileSink {
    type Error = io::Error;
    async fn write_chunk<'a>(&'a mut self, bytes: &'a [u8]) -> io::Result<usize> {
        let len = BlockingStreamSink::write_chunk(self, bytes)?;
        if self.pause == Pause::Write {
            core::future::pending::<()>().await;
        }
        Ok(len)
    }
    async fn commit(&mut self) -> io::Result<()> {
        if self.pause == Pause::Commit {
            core::future::pending::<()>().await;
        }
        BlockingStreamSink::commit(self)
    }
    fn abort(&mut self, partial: StreamPartialState) {
        BlockingStreamSink::abort(self, partial);
    }
}
impl Drop for FileSink {
    fn drop(&mut self) {
        self.file.take();
        if self.committed || self.aborts == 0 {
            fs::remove_file(&self.temporary).fixture("remove tentative name");
        }
    }
}

fn download() -> ArtifactDownload<'static> {
    // Independent public SHA-256 known answer for "abc"; not computed from input.
    let expected = [
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22,
        0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00,
        0x15, 0xad,
    ];
    ArtifactDownload::new(
        CrateName::new("serde").fixture("crate"),
        Version::new("1.0.0").fixture("version"),
        3,
        expected,
        StreamLimits::new(3, 2, 8, 32, 2).fixture("limits"),
    )
    .fixture("download")
}

fn execute(
    mode: u8,
    transport: &Transport,
    sink: &mut FileSink,
    scratch: &mut [u8],
) -> Result<StreamCompletion, ArtifactError> {
    let checksum = crate::downloads::Sha256Checksum::new();
    match mode {
        0 => download().execute(transport, identity(), sink, checksum, scratch),
        1 => ready(send(download().execute_async(
            transport,
            identity(),
            sink,
            checksum,
            scratch,
        ))),
        _ => ready(download().execute_local(transport, identity(), sink, checksum, scratch)),
    }
}
