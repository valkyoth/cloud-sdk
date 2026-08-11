extern crate std;

use core::cell::Cell;

use super::JsonError;

std::thread_local! {
    static FAIL_NEXT: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn checkpoint() -> Result<(), JsonError> {
    FAIL_NEXT.with(|flag| {
        if flag.replace(false) {
            Err(JsonError::Allocation)
        } else {
            Ok(())
        }
    })
}

pub(crate) fn with_next_failure<R>(operation: impl FnOnce() -> R) -> R {
    FAIL_NEXT.with(|flag| {
        assert!(!flag.replace(true), "nested allocation failure injection");
        let _reset = Reset(flag);
        operation()
    })
}

struct Reset<'a>(&'a Cell<bool>);

impl Drop for Reset<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}
