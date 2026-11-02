//! Exponential moving average (EMA) for latency / rate smoothing.

/// `value = alpha * sample + (1 - alpha) * value`
#[derive(Debug, Clone, Copy)]
pub struct Ema {
    alpha: f64,
    value: Option<f64>,
    count: u64,
}

impl Ema {
    /// `alpha` in (0, 1]; higher = more weight on recent samples.
    pub fn new(alpha: f64) -> Self {
        Self {
            alpha: alpha.clamp(0.01, 1.0),
            value: None,
            count: 0,
        }
    }

    pub fn observe(&mut self, sample: f64) -> f64 {
        self.count = self.count.saturating_add(1);
        let v = match self.value {
            None => sample,
            Some(prev) => self.alpha * sample + (1.0 - self.alpha) * prev,
        };
        self.value = Some(v);
        v
    }

    pub fn get(&self) -> Option<f64> {
        self.value
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn reset(&mut self) {
        self.value = None;
        self.count = 0;
    }
}

impl Default for Ema {
    fn default() -> Self {
        Self::new(0.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_toward_samples() {
        let mut e = Ema::new(0.5);
        e.observe(10.0);
        let v = e.observe(20.0);
        assert!((v - 15.0).abs() < 1e-9);
    }
}
