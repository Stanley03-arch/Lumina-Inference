//! Priority queue for pending inference requests.

use crate::batch::InferenceRequest;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Higher value = higher priority (processed first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Priority(pub u8);

impl Priority {
    pub const LOW: Self = Self(0);
    pub const NORMAL: Self = Self(100);
    pub const HIGH: Self = Self(200);
    pub const CRITICAL: Self = Self(255);
}

#[derive(Debug)]
struct Item {
    priority: Priority,
    seq: u64,
    request: InferenceRequest,
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.seq == other.seq
    }
}
impl Eq for Item {}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first; FIFO within same priority (lower seq first → reverse seq)
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.seq.cmp(&self.seq))
    }
}

/// Max-heap of requests ordered by priority then arrival.
#[derive(Debug, Default)]
pub struct PriorityQueue {
    heap: BinaryHeap<Item>,
    next_seq: u64,
    max: usize,
}

impl PriorityQueue {
    pub fn new(max: usize) -> Self {
        Self {
            heap: BinaryHeap::new(),
            next_seq: 0,
            max: max.max(1),
        }
    }

    pub fn push(&mut self, priority: Priority, request: InferenceRequest) -> bool {
        if self.heap.len() >= self.max {
            return false;
        }
        let seq = self.next_seq;
        self.next_seq = self.next_seq.wrapping_add(1);
        self.heap.push(Item {
            priority,
            seq,
            request,
        });
        true
    }

    pub fn pop(&mut self) -> Option<InferenceRequest> {
        self.heap.pop().map(|i| i.request)
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn clear(&mut self) {
        self.heap.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_before_low() {
        let mut q = PriorityQueue::new(10);
        q.push(Priority::LOW, InferenceRequest::new("low", "m", vec![]));
        q.push(Priority::HIGH, InferenceRequest::new("high", "m", vec![]));
        q.push(
            Priority::NORMAL,
            InferenceRequest::new("mid", "m", vec![]),
        );
        assert_eq!(q.pop().unwrap().id, "high");
        assert_eq!(q.pop().unwrap().id, "mid");
        assert_eq!(q.pop().unwrap().id, "low");
    }
}
