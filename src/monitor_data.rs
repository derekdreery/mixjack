use parking_lot::{Condvar, Mutex};
use std::sync::{atomic::AtomicU64, Arc};

/// A way of sharing data between the RT thread and the UI thread such that the RT thread never
/// blocks.
///
/// Only works with 1 waiter & 1 updater.
#[derive(Clone)]
pub struct MonitorData<T> {
    inner: Arc<Shared<T>>,
}

struct Shared<T> {
    /// data (usize is generation number)
    data: Mutex<Inner<T>>,
    /// waker
    waker: Condvar,
}

struct Inner<T> {
    value: T,
    new_data: bool,
    shutdown: bool,
}

impl<T> MonitorData<T> {
    pub fn new(inner: T) -> Self {
        MonitorData {
            inner: Arc::new(Shared {
                data: Mutex::new(Inner {
                    value: inner,
                    new_data: false,
                    shutdown: false,
                }),
                waker: Condvar::new(),
            }),
        }
    }

    /// Add an update and inc. the gen number. If the mutex is locked then skip over.
    pub fn update(&self, cb: impl FnOnce(&mut T)) {
        let mut data = match self.inner.data.try_lock() {
            Some(lock) => lock,
            // we couldn't get a lock, try again on next frame
            None => return,
        };
        data.new_data = true;
        cb(&mut data.value);
        self.inner.waker.notify_one();
    }

    /// Wait until prev_gen < current generation, then update prev_gen to gen and call cb.
    pub fn on_changed(&self, mut cb: impl FnMut(&T)) {
        let mut data = self.inner.data.lock();
        loop {
            if data.shutdown {
                break;
            }
            if !data.new_data {
                self.inner.waker.wait(&mut data);
            }
            cb(&data.value);
            data.new_data = false;
        }
    }
}
