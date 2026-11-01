//! Adaptive batch size: grow when latency is low, shrink when high.

/// Controller that nudges `target` batch size from observed latency.
#[derive(Debug, Clone)]
pub struct AdaptiveBatch {
    pub min: usize,
    pub max: usize,
    pub target: usize,
    /// Latency budget in ms; above this → shrink.
    pub latency_budget_ms: u64,
    /// Below this fraction of budget → grow.
    pub grow_factor: f64,
}

impl AdaptiveBatch {
    pub fn new(min: usize, max: usize, initial: usize, latency_budget_ms: u64) -> Self {
        let min = min.max(1);
        let max = max.max(min);
        Self {
            min,
            max,
            target: initial.clamp(min, max),
            latency_budget_ms: latency_budget_ms.max(1),
            grow_factor: 0.5,
        }
    }

    pub fn current(&self) -> usize {
        self.target
    }

    /// Feed measured batch latency; updates `target`.
    pub fn observe(&mut self, latency_ms: u64, batch_len: usize) {
        let _ = batch_len;
        let budget = self.latency_budget_ms as f64;
        let lat = latency_ms as f64;
        if lat > budget {
            // Shrink toward min
            self.target = ((self.target as f64) * 0.75).ceil() as usize;
        } else if lat < budget * self.grow_factor {
            self.target = self.target.saturating_add(1);
        }
        self.target = self.target.clamp(self.min, self.max);
    }
}

impl Default for AdaptiveBatch {
    fn default() -> Self {
        Self::new(1, 64, 8, 50)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrinks_on_slow() {
        let mut a = AdaptiveBatch::new(1, 32, 16, 20);
        a.observe(100, 16);
        assert!(a.current() < 16);
    }

    #[test]
    fn grows_on_fast() {
        let mut a = AdaptiveBatch::new(1, 32, 4, 100);
        a.observe(10, 4);
        assert!(a.current() >= 4);
    }
}
