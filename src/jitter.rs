//! Jitter helpers for retry / gossip intervals.

use std::time::Duration;

/// Full jitter: uniform in `[0, base]`.
pub fn full_jitter(base: Duration, seed: u64) -> Duration {
    if base.is_zero() {
        return Duration::ZERO;
    }
    let nanos = base.as_nanos() as u64;
    let j = splitmix64(seed) % nanos.saturating_add(1);
    Duration::from_nanos(j)
}

/// Equal jitter: `base/2 + uniform(0, base/2)`.
pub fn equal_jitter(base: Duration, seed: u64) -> Duration {
    if base.is_zero() {
        return Duration::ZERO;
    }
    let half = base / 2;
    half + full_jitter(base - half, seed)
}

/// Decorrelated-style: min(cap, rand(base, prev*3)).
pub fn decorrelated(base: Duration, prev: Duration, cap: Duration, seed: u64) -> Duration {
    let lo = base.as_nanos() as u64;
    let hi = prev.as_nanos().saturating_mul(3) as u64;
    let (lo, hi) = if hi < lo { (hi, lo) } else { (lo, hi) };
    let span = hi.saturating_sub(lo).saturating_add(1);
    let v = lo + splitmix64(seed) % span;
    Duration::from_nanos(v).min(cap)
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_in_range() {
        let b = Duration::from_millis(100);
        for s in 0..20 {
            let j = full_jitter(b, s);
            assert!(j <= b);
        }
    }
}
