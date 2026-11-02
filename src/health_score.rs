//! Composite 0–100 health score for workers.

/// Blend of success rate and latency vs budget.
#[derive(Debug, Clone, Copy)]
pub struct HealthScore {
    pub success_weight: f64,
    pub latency_weight: f64,
    pub latency_budget_ms: f64,
}

impl Default for HealthScore {
    fn default() -> Self {
        Self {
            success_weight: 0.7,
            latency_weight: 0.3,
            latency_budget_ms: 100.0,
        }
    }
}

impl HealthScore {
    /// `success_rate` in 0..1, `avg_latency_ms` ≥ 0 → score 0..100.
    pub fn score(&self, success_rate: f64, avg_latency_ms: f64) -> f64 {
        let sr = success_rate.clamp(0.0, 1.0);
        let lat_ratio = if self.latency_budget_ms <= 0.0 {
            1.0
        } else {
            (1.0 - (avg_latency_ms / self.latency_budget_ms)).clamp(0.0, 1.0)
        };
        let w_s = self.success_weight;
        let w_l = self.latency_weight;
        let w = (w_s + w_l).max(1e-9);
        ((w_s * sr + w_l * lat_ratio) / w * 100.0).clamp(0.0, 100.0)
    }

    pub fn is_healthy(&self, success_rate: f64, avg_latency_ms: f64, min_score: f64) -> bool {
        self.score(success_rate, avg_latency_ms) >= min_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect() {
        let h = HealthScore::default();
        assert!((h.score(1.0, 0.0) - 100.0).abs() < 1e-6);
    }
}
