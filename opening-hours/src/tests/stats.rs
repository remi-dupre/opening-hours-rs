//! Generate some runtime stats, only during tests

use std::cell::RefCell;

thread_local! {
    /// Compute stats for current thread.
    static THREAD_TEST_STATS: RefCell<TestStats> = RefCell::default();
    /// Used as a lock to ensure a single consumer can be waiting for
    /// statistics at single time.
    static THREAD_LOCK: RefCell<()> = RefCell::default();
}

#[derive(Default)]
pub(crate) struct TestStats {
    /// Number of times a schedule has been generated for a whole expression.
    pub(crate) count_generated_schedules: u64,
}

impl TestStats {
    /// Compute stats for the duration of the wrapped function
    pub(crate) fn watch<F: FnOnce()>(func: F) -> Self {
        THREAD_LOCK.with_borrow_mut(|_| {
            THREAD_TEST_STATS.take();
            func();
            THREAD_TEST_STATS.take()
        })
    }
}

pub(crate) mod notify {
    pub(crate) fn generated_schedule() {
        super::THREAD_TEST_STATS.with_borrow_mut(|s| s.count_generated_schedules += 1)
    }
}
