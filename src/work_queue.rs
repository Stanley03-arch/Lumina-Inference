//! Bounded FIFO work queue for inference requests.

use crate::batch::InferenceRequest;
use crate::error::{LuminaError, Result};
use std::collections::VecDeque;

/// Simple bounded queue (single-threaded / external mutex).
#[derive(Debug, Default)]
pub struct WorkQueue {
    items: VecDeque<InferenceRequest>,
    capacity: usize,
}

impl WorkQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            items: VecDeque::with_capacity(capacity.max(1)),
            capacity: capacity.max(1),
        }
    }

    pub fn push(&mut self, req: InferenceRequest) -> Result<()> {
        if self.items.len() >= self.capacity {
            return Err(LuminaError::BatchOverflow {
                max: self.capacity,
            });
        }
        self.items.push_back(req);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<InferenceRequest> {
        self.items.pop_front()
    }

    /// Pop up to `n` items.
    pub fn pop_batch(&mut self, n: usize) -> Vec<InferenceRequest> {
        let mut out = Vec::with_capacity(n.min(self.items.len()));
        for _ in 0..n {
            match self.items.pop_front() {
                Some(r) => out.push(r),
                None => break,
            }
        }
        out
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.items.len() >= self.capacity
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_and_cap() {
        let mut q = WorkQueue::new(2);
        q.push(InferenceRequest::new("1", "m", vec![])).unwrap();
        q.push(InferenceRequest::new("2", "m", vec![])).unwrap();
        assert!(q.push(InferenceRequest::new("3", "m", vec![])).is_err());
        assert_eq!(q.pop().unwrap().id, "1");
        assert_eq!(q.pop_batch(10).len(), 1);
    }
}
