//! Load-aware scheduling over a [`WorkerRegistry`].
//!
//! Tracks in-flight requests per node and prefers peers with spare capacity.

use crate::discovery::{WorkerNode, WorkerRegistry};
use std::collections::HashMap;
use std::cmp::Ordering;

/// How to choose among healthy candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SchedulePolicy {
    /// Classic round-robin (delegates to registry `pick`).
    #[default]
    RoundRobin,
    /// Prefer the node with the lowest `in_flight / capacity` ratio.
    LeastLoaded,
    /// Weighted by remaining capacity (`capacity - in_flight`).
    WeightedCapacity,
}

/// Tracks outstanding work per worker id.
#[derive(Debug, Default, Clone)]
pub struct LoadTracker {
    in_flight: HashMap<String, u32>,
}

impl LoadTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn in_flight(&self, id: &str) -> u32 {
        self.in_flight.get(id).copied().unwrap_or(0)
    }

    /// Record that a request was dispatched to `id`.
    pub fn acquire(&mut self, id: &str) {
        *self.in_flight.entry(id.to_string()).or_insert(0) += 1;
    }

    /// Record that a request finished on `id`.
    pub fn release(&mut self, id: &str) {
        if let Some(c) = self.in_flight.get_mut(id) {
            *c = c.saturating_sub(1);
            if *c == 0 {
                self.in_flight.remove(id);
            }
        }
    }

    pub fn load_ratio(&self, node: &WorkerNode) -> f64 {
        let inflight = self.in_flight(&node.id) as f64;
        let cap = node.capacity.max(1) as f64;
        inflight / cap
    }

    pub fn remaining_capacity(&self, node: &WorkerNode) -> u32 {
        node.capacity.saturating_sub(self.in_flight(&node.id))
    }
}

/// Scheduler that combines registry membership with load tracking.
#[derive(Debug, Clone)]
pub struct Scheduler {
    pub policy: SchedulePolicy,
    load: LoadTracker,
    rr: usize,
}

impl Scheduler {
    pub fn new(policy: SchedulePolicy) -> Self {
        Self {
            policy,
            load: LoadTracker::new(),
            rr: 0,
        }
    }

    pub fn load(&self) -> &LoadTracker {
        &self.load
    }

    pub fn load_mut(&mut self) -> &mut LoadTracker {
        &mut self.load
    }

    /// Pick a healthy node for `model` using the configured policy.
    /// Does **not** increment in-flight; call [`Self::acquire`] after dispatch.
    pub fn pick(&mut self, registry: &WorkerRegistry, model: &str) -> Option<WorkerNode> {
        let mut candidates: Vec<WorkerNode> = registry
            .healthy_for_model(model)
            .into_iter()
            .cloned()
            .collect();
        if candidates.is_empty() {
            return None;
        }

        match self.policy {
            SchedulePolicy::RoundRobin => {
                let idx = self.rr % candidates.len();
                self.rr = self.rr.wrapping_add(1);
                Some(candidates[idx].clone())
            }
            SchedulePolicy::LeastLoaded => {
                candidates.sort_by(|a, b| {
                    self.load
                        .load_ratio(a)
                        .partial_cmp(&self.load.load_ratio(b))
                        .unwrap_or(Ordering::Equal)
                        .then_with(|| b.capacity.cmp(&a.capacity))
                        .then_with(|| a.id.cmp(&b.id))
                });
                Some(candidates[0].clone())
            }
            SchedulePolicy::WeightedCapacity => {
                let with_room: Vec<_> = candidates
                    .iter()
                    .filter(|n| self.load.remaining_capacity(n) > 0)
                    .cloned()
                    .collect();
                let pool = if with_room.is_empty() {
                    candidates
                } else {
                    with_room
                };
                let total: u64 = pool
                    .iter()
                    .map(|n| self.load.remaining_capacity(n).max(1) as u64)
                    .sum();
                if total == 0 {
                    return pool.into_iter().next();
                }
                let mut slot = (self.rr as u64) % total;
                self.rr = self.rr.wrapping_add(1);
                for n in &pool {
                    let w = self.load.remaining_capacity(n).max(1) as u64;
                    if slot < w {
                        return Some(n.clone());
                    }
                    slot -= w;
                }
                pool.into_iter().next()
            }
        }
    }

    /// Pick and immediately mark in-flight.
    pub fn pick_and_acquire(
        &mut self,
        registry: &WorkerRegistry,
        model: &str,
    ) -> Option<WorkerNode> {
        let node = self.pick(registry, model)?;
        self.load.acquire(&node.id);
        Some(node)
    }

    pub fn acquire(&mut self, id: &str) {
        self.load.acquire(id);
    }

    pub fn release(&mut self, id: &str) {
        self.load.release(id);
    }

    pub fn lease<'a>(&'a mut self, node: &WorkerNode) -> Lease<'a> {
        self.load.acquire(&node.id);
        Lease {
            scheduler: self,
            id: node.id.clone(),
        }
    }
}

/// RAII guard that releases load when dropped.
pub struct Lease<'a> {
    scheduler: &'a mut Scheduler,
    id: String,
}

impl Drop for Lease<'_> {
    fn drop(&mut self) {
        self.scheduler.release(&self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg_two() -> WorkerRegistry {
        let mut reg = WorkerRegistry::new();
        let mut a = WorkerNode::new("a", "http://a:1");
        a.capacity = 2;
        a.models = vec!["m".into()];
        let mut b = WorkerNode::new("b", "http://b:1");
        b.capacity = 8;
        b.models = vec!["m".into()];
        reg.register(a);
        reg.register(b);
        reg
    }

    #[test]
    fn least_loaded_prefers_idle() {
        let reg = reg_two();
        let mut sched = Scheduler::new(SchedulePolicy::LeastLoaded);
        sched.acquire("a");
        sched.acquire("a");
        let n = sched.pick(&reg, "m").unwrap();
        assert_eq!(n.id, "b");
    }

    #[test]
    fn weighted_prefers_higher_capacity() {
        let reg = reg_two();
        let mut sched = Scheduler::new(SchedulePolicy::WeightedCapacity);
        let mut counts = HashMap::<String, u32>::new();
        for _ in 0..100 {
            let n = sched.pick(&reg, "m").unwrap();
            *counts.entry(n.id).or_insert(0) += 1;
        }
        assert!(counts.get("b").copied().unwrap_or(0) > counts.get("a").copied().unwrap_or(0));
    }

    #[test]
    fn lease_releases() {
        let mut sched = Scheduler::new(SchedulePolicy::LeastLoaded);
        let n = WorkerNode::new("x", "http://x");
        {
            let _lease = sched.lease(&n);
            assert_eq!(sched.load().in_flight("x"), 1);
        }
        assert_eq!(sched.load().in_flight("x"), 0);
    }
}
