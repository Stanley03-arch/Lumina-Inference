//! Human-readable byte sizes.

/// Format bytes as `1.5 KiB`, `2.0 MiB`, etc. (binary units).
pub fn format_bytes(n: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    if n < 1024 {
        return format!("{n} B");
    }
    let mut v = n as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    format!("{v:.1} {}", UNITS[i])
}

/// Parse simple sizes like `16mb`, `1GiB`, `512` (bytes).
pub fn parse_bytes(s: &str) -> Option<u64> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }
    let (num, mult) = if let Some(rest) = s.strip_suffix("pib").or_else(|| s.strip_suffix('p')) {
        (rest, 1u64 << 50)
    } else if let Some(rest) = s.strip_suffix("tib").or_else(|| s.strip_suffix('t')) {
        (rest, 1u64 << 40)
    } else if let Some(rest) = s.strip_suffix("gib").or_else(|| s.strip_suffix('g')) {
        (rest, 1u64 << 30)
    } else if let Some(rest) = s
        .strip_suffix("mib")
        .or_else(|| s.strip_suffix("mb"))
        .or_else(|| s.strip_suffix('m'))
    {
        (rest, 1u64 << 20)
    } else if let Some(rest) = s
        .strip_suffix("kib")
        .or_else(|| s.strip_suffix("kb"))
        .or_else(|| s.strip_suffix('k'))
    {
        (rest, 1u64 << 10)
    } else if let Some(rest) = s.strip_suffix('b') {
        (rest, 1)
    } else {
        (s.as_str(), 1)
    };
    let num = num.trim();
    if let Ok(v) = num.parse::<u64>() {
        return v.checked_mul(mult);
    }
    if let Ok(v) = num.parse::<f64>() {
        return Some((v * mult as f64) as u64);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_and_parse() {
        assert_eq!(format_bytes(1536), "1.5 KiB");
        assert_eq!(parse_bytes("16mb"), Some(16 * 1024 * 1024));
        assert_eq!(parse_bytes("1GiB"), Some(1 << 30));
    }
}
