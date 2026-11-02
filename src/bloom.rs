//! Minimal Bloom filter for approximate membership (e.g. request-id dedup).

/// Fixed-size bit Bloom filter (false positives possible; no false negatives).
#[derive(Debug, Clone)]
pub struct BloomFilter {
    bits: Vec<u64>,
    k: usize,
}

impl BloomFilter {
    /// `bit_count` rounded up to multiples of 64; `k` hash functions.
    pub fn new(bit_count: usize, k: usize) -> Self {
        let words = (bit_count.max(64) + 63) / 64;
        Self {
            bits: vec![0u64; words],
            k: k.max(1).min(16),
        }
    }

    fn hashes(&self, data: &[u8]) -> impl Iterator<Item = usize> + '_ {
        let h1 = fnv1a(data);
        let h2 = h1.rotate_left(13).wrapping_mul(0x9e3779b97f4a7c15);
        let nbits = self.bits.len() * 64;
        (0..self.k).map(move |i| {
            let h = h1.wrapping_add((i as u64).wrapping_mul(h2));
            (h as usize) % nbits
        })
    }

    pub fn insert(&mut self, data: &[u8]) {
        for idx in self.hashes(data) {
            let word = idx / 64;
            let bit = idx % 64;
            self.bits[word] |= 1u64 << bit;
        }
    }

    pub fn insert_str(&mut self, s: &str) {
        self.insert(s.as_bytes());
    }

    pub fn might_contain(&self, data: &[u8]) -> bool {
        self.hashes(data).all(|idx| {
            let word = idx / 64;
            let bit = idx % 64;
            (self.bits[word] & (1u64 << bit)) != 0
        })
    }

    pub fn might_contain_str(&self, s: &str) -> bool {
        self.might_contain(s.as_bytes())
    }

    pub fn clear(&mut self) {
        for w in &mut self.bits {
            *w = 0;
        }
    }
}

impl Default for BloomFilter {
    fn default() -> Self {
        Self::new(4096, 4)
    }
}

fn fnv1a(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_check() {
        let mut b = BloomFilter::new(1024, 3);
        b.insert_str("req-1");
        assert!(b.might_contain_str("req-1"));
        // Unseen keys are usually false; not asserted (FP possible).
    }
}
