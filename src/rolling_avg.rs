//! Fixed-window rolling average of f64 samples.

use std::collections::VecDeque;

/// Mean of the last `capacity` samples.
#[derive(Debug, Clone)]
pub struct RollingAvg {
    samples: VecDeque<f64>,
    capacity: usize,
    sum: f64,
}

impl RollingAvg {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            samples: VecDeque::with_capacity(capacity),
            capacity,
            sum: 0.0,
        }
    }

    pub fn push(&mut self, value: f64) -> f64 {
        if self.samples.len() == self.capacity {
            if let Some(old) = self.samples.pop_front() {
                self.sum -= old;
            }
        }
        self.samples.push_back(value);
        self.sum += value;
        self.mean()
    }

    pub fn mean(&self) -> f64 {
        if self.samples.is_empty() {
            0.0
        } else {
            self.sum / self.samples.len() as f64
        }
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn clear(&mut self) {
        self.samples.clear();
        self.sum = 0.0;
    }
}

impl Default for RollingAvg {
    fn default() -> Self {
        Self::new(32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn averages() {
        let mut r = RollingAvg::new(3);
        r.push(1.0);
        r.push(2.0);
        r.push(3.0);
        assert!((r.mean() - 2.0).abs() < 1e-9);
        r.push(6.0); // drops 1 → 2,3,6
        assert!((r.mean() - 3.666666).abs() < 0.01);
    }
}
