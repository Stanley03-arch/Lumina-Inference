//! Atomic once-flag (run initialization at most once).

use std::sync::atomic::{AtomicBool, Ordering};

/// Set-once boolean; useful for one-shot warm-up / init.
#[derive(Debug, Default)]
pub struct OnceFlag {
    done: AtomicBool,
}

impl OnceFlag {
    pub fn new() -> Self {
        Self {
            done: AtomicBool::new(false),
        }
    }

    pub fn is_set(&self) -> bool {
        self.done.load(Ordering::Acquire)
    }

    /// Returns `true` if this call transitioned false → true.
    pub fn set(&self) -> bool {
        self.done
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    /// Run `f` only the first time; returns whether `f` ran.
    pub fn call_once<F: FnOnce()>(&self, f: F) -> bool {
        if self.set() {
            f();
            true
        } else {
            false
        }
    }

    pub fn reset(&self) {
        self.done.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_once() {
        let f = OnceFlag::new();
        let mut n = 0;
        assert!(f.call_once(|| n += 1));
        assert!(!f.call_once(|| n += 1));
        assert_eq!(n, 1);
    }
}
