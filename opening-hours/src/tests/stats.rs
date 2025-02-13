use std::cell::RefCell;

thread_local! {
    static THREAD_TEST_STATS: RefCell<TestStats> = RefCell::default();
    static THREAD_LOCK: RefCell<()> = RefCell::default();
}

#[derive(Default)]
pub(crate) struct TestStats {
    pub(crate) count_generated_schedules: u64,
}

impl TestStats {
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
