//! In-flight concurrency limiter (semaphore-style backpressure).

use crate::error::{LuminaError, Result};
use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug)]
struct State {
    in_flight: usize,
    max: usize,
}

/// Limits concurrent work; blocks or rejects when full.
#[derive(Debug, Clone)]
pub struct Backpressure {
    inner: Arc<(Mutex<State>, Condvar)>,
}

impl Backpressure {
    pub fn new(max: usize) -> Self {
        Self {
            inner: Arc::new((
                Mutex::new(State {
                    in_flight: 0,
                    max: max.max(1),
                }),
                Condvar::new(),
            )),
        }
    }

    pub fn max(&self) -> usize {
        self.inner.0.lock().unwrap().max
    }

    pub fn in_flight(&self) -> usize {
        self.inner.0.lock().unwrap().in_flight
    }

    /// Non-blocking acquire.
    pub fn try_acquire(&self) -> Result<Permit> {
        let mut st = self.inner.0.lock().unwrap();
        if st.in_flight >= st.max {
            return Err(LuminaError::RateLimited);
        }
        st.in_flight += 1;
        Ok(Permit {
            bp: self.clone(),
        })
    }

    /// Blocking acquire (waits until a slot frees).
    pub fn acquire(&self) -> Permit {
        let (lock, cv) = &*self.inner;
        let mut st = lock.lock().unwrap();
        while st.in_flight >= st.max {
            st = cv.wait(st).unwrap();
        }
        st.in_flight += 1;
        Permit {
            bp: self.clone(),
        }
    }

    fn release(&self) {
        let (lock, cv) = &*self.inner;
        let mut st = lock.lock().unwrap();
        st.in_flight = st.in_flight.saturating_sub(1);
        cv.notify_one();
    }
}

/// RAII permit; releases on drop.
pub struct Permit {
    bp: Backpressure,
}

impl Drop for Permit {
    fn drop(&mut self) {
        self.bp.release();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_when_full() {
        let bp = Backpressure::new(1);
        let _p = bp.try_acquire().unwrap();
        assert!(bp.try_acquire().is_err());
    }
}
