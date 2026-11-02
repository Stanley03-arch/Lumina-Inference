//! Countdown latch — wait until N parties arrive.

use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// Threads/tasks call `count_down`; waiters block until count hits zero.
#[derive(Debug, Clone)]
pub struct CountDownLatch {
    inner: Arc<(Mutex<usize>, Condvar)>,
}

impl CountDownLatch {
    pub fn new(count: usize) -> Self {
        Self {
            inner: Arc::new((Mutex::new(count), Condvar::new())),
        }
    }

    pub fn count_down(&self) {
        let (lock, cv) = &*self.inner;
        let mut c = lock.lock().unwrap();
        if *c > 0 {
            *c -= 1;
            if *c == 0 {
                cv.notify_all();
            }
        }
    }

    pub fn remaining(&self) -> usize {
        *self.inner.0.lock().unwrap()
    }

    pub fn wait(&self) {
        let (lock, cv) = &*self.inner;
        let mut c = lock.lock().unwrap();
        while *c > 0 {
            c = cv.wait(c).unwrap();
        }
    }

    /// Returns `true` if reached zero within timeout.
    pub fn wait_timeout(&self, timeout: Duration) -> bool {
        let (lock, cv) = &*self.inner;
        let mut c = lock.lock().unwrap();
        let start = std::time::Instant::now();
        while *c > 0 {
            let left = timeout.saturating_sub(start.elapsed());
            if left.is_zero() {
                return false;
            }
            let (guard, _) = cv.wait_timeout(c, left).unwrap();
            c = guard;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_immediate() {
        let l = CountDownLatch::new(0);
        assert!(l.wait_timeout(Duration::from_millis(10)));
    }

    #[test]
    fn counts_down() {
        let l = CountDownLatch::new(2);
        l.count_down();
        assert_eq!(l.remaining(), 1);
        l.count_down();
        assert!(l.wait_timeout(Duration::from_millis(50)));
    }
}
