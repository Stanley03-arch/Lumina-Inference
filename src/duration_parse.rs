//! Parse human duration strings for config / env vars.

use std::time::Duration;

/// Parse `30ms`, `5s`, `2m`, `1h`, bare milliseconds number.
pub fn parse_duration(s: &str) -> Option<Duration> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }
    if let Some(rest) = s.strip_suffix("ms") {
        return rest.trim().parse::<u64>().ok().map(Duration::from_millis);
    }
    if let Some(rest) = s.strip_suffix("us").or_else(|| s.strip_suffix("µs")) {
        return rest.trim().parse::<u64>().ok().map(Duration::from_micros);
    }
    if let Some(rest) = s.strip_suffix('s') {
        let rest = rest.trim();
        if let Ok(v) = rest.parse::<u64>() {
            return Some(Duration::from_secs(v));
        }
        if let Ok(v) = rest.parse::<f64>() {
            return Some(Duration::from_secs_f64(v));
        }
        return None;
    }
    if let Some(rest) = s.strip_suffix('m') {
        return rest
            .trim()
            .parse::<u64>()
            .ok()
            .map(|m| Duration::from_secs(m.saturating_mul(60)));
    }
    if let Some(rest) = s.strip_suffix('h') {
        return rest
            .trim()
            .parse::<u64>()
            .ok()
            .map(|h| Duration::from_secs(h.saturating_mul(3600)));
    }
    // bare integer → milliseconds
    s.parse::<u64>().ok().map(Duration::from_millis)
}

/// Read env var as duration, or default.
pub fn duration_from_env(key: &str, default: Duration) -> Duration {
    std::env::var(key)
        .ok()
        .and_then(|v| parse_duration(&v))
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_units() {
        assert_eq!(parse_duration("30ms"), Some(Duration::from_millis(30)));
        assert_eq!(parse_duration("2s"), Some(Duration::from_secs(2)));
        assert_eq!(parse_duration("1m"), Some(Duration::from_secs(60)));
        assert_eq!(parse_duration("100"), Some(Duration::from_millis(100)));
    }
}
