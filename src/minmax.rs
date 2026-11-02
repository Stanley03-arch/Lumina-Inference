//! Running min / max / mean statistics.

/// Online min, max, sum, count.
#[derive(Debug, Clone, Copy, Default)]
pub struct MinMax {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub sum: f64,
    pub count: u64,
}

impl MinMax {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn observe(&mut self, v: f64) {
        self.min = Some(self.min.map_or(v, |m| m.min(v)));
        self.max = Some(self.max.map_or(v, |m| m.max(v)));
        self.sum += v;
        self.count = self.count.saturating_add(1);
    }

    pub fn mean(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks() {
        let mut m = MinMax::new();
        m.observe(10.0);
        m.observe(2.0);
        m.observe(8.0);
        assert_eq!(m.min, Some(2.0));
        assert_eq!(m.max, Some(10.0));
        assert!((m.mean().unwrap() - 6.666).abs() < 0.01);
    }
}
