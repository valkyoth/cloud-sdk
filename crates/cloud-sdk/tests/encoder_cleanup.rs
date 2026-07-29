//! Panic-unwind cleanup regressions for public snapshot encoding.

use std::sync::atomic::{AtomicUsize, Ordering};

use cloud_sdk::buffer::{SnapshotEncoder, encode_snapshot};

#[derive(Clone, Copy)]
struct TestError;

static WRITE_PASS: AtomicUsize = AtomicUsize::new(0);
static VERIFY_PASS: AtomicUsize = AtomicUsize::new(0);

fn panic_on_write(
    snapshot: &str,
    encoder: &mut SnapshotEncoder<'_, TestError>,
) -> Result<(), TestError> {
    let pass = WRITE_PASS.fetch_add(1, Ordering::SeqCst);
    encoder.string(snapshot)?;
    if pass == 1 {
        std::panic::resume_unwind(Box::new("write unwind"));
    }
    Ok(())
}

fn panic_on_verify(
    snapshot: &str,
    encoder: &mut SnapshotEncoder<'_, TestError>,
) -> Result<(), TestError> {
    let pass = VERIFY_PASS.fetch_add(1, Ordering::SeqCst);
    encoder.string(snapshot)?;
    if pass == 2 {
        std::panic::resume_unwind(Box::new("verification unwind"));
    }
    Ok(())
}

#[test]
fn panic_during_write_clears_the_exact_admitted_prefix() {
    WRITE_PASS.store(0, Ordering::SeqCst);
    let mut output = [0xA5_u8; 10];
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = encode_snapshot("secret", &mut output, TestError, panic_on_write);
    }));

    assert!(unwind.is_err());
    assert_eq!(output.get(..6), Some([0_u8; 6].as_slice()));
    assert_eq!(output.get(6..), Some([0xA5_u8; 4].as_slice()));
}

#[test]
fn panic_during_verification_clears_the_exact_admitted_prefix() {
    VERIFY_PASS.store(0, Ordering::SeqCst);
    let mut output = [0xA5_u8; 10];
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = encode_snapshot("secret", &mut output, TestError, panic_on_verify);
    }));

    assert!(unwind.is_err());
    assert_eq!(output.get(..6), Some([0_u8; 6].as_slice()));
    assert_eq!(output.get(6..), Some([0xA5_u8; 4].as_slice()));
}
