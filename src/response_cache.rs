//! LRU cache of inference responses by content fingerprint.

use crate::batch::{InferenceRequest, InferenceResponse};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Entry {
    response: InferenceResponse,
    last: Instant,
}

/// Cache identical model+input requests for a short TTL.
#[derive(Debug, Clone)]
pub struct ResponseCache {
    map: HashMap<u64, Entry>,
    max: usize,
    ttl: Duration,
}

impl ResponseCache {
    pub fn new(max: usize, ttl: Duration) -> Self {
        Self {
            map: HashMap::new(),
            max: max.max(1),
            ttl,
        }
    }

    /// FNV-1a style fingerprint of model + primary input + named inputs.
    pub fn fingerprint(req: &InferenceRequest) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in req.model.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h ^= 0xff;
        h = h.wrapping_mul(0x100000001b3);
        for b in &req.input {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        for (name, data) in &req.named_inputs.tensors {
            for b in name.bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
            for b in data {
                h ^= *b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
        }
        h
    }

    pub fn get(&mut self, req: &InferenceRequest) -> Option<InferenceResponse> {
        self.evict_expired();
        let key = Self::fingerprint(req);
        if let Some(e) = self.map.get_mut(&key) {
            e.last = Instant::now();
            let mut resp = e.response.clone();
            // Keep caller id
            resp.id = req.id.clone();
            return Some(resp);
        }
        None
    }

    pub fn insert(&mut self, req: &InferenceRequest, mut response: InferenceResponse) {
        self.evict_expired();
        if self.map.len() >= self.max {
            if let Some(oldest) = self
                .map
                .iter()
                .min_by_key(|(_, e)| e.last)
                .map(|(k, _)| *k)
            {
                self.map.remove(&oldest);
            }
        }
        // Store without binding to a single request id
        response.id.clear();
        self.map.insert(
            Self::fingerprint(req),
            Entry {
                response,
                last: Instant::now(),
            },
        );
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    fn evict_expired(&mut self) {
        let ttl = self.ttl;
        self.map.retain(|_, e| e.last.elapsed() < ttl);
    }
}

impl Default for ResponseCache {
    fn default() -> Self {
        Self::new(256, Duration::from_secs(30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hits_same_input() {
        let mut c = ResponseCache::new(8, Duration::from_secs(60));
        let req = InferenceRequest::new("1", "m", vec![1, 2, 3]);
        let resp = InferenceResponse::new("1".into(), "m".into(), vec![9], 1);
        c.insert(&req, resp);
        let req2 = InferenceRequest::new("2", "m", vec![1, 2, 3]);
        let hit = c.get(&req2).unwrap();
        assert_eq!(hit.id, "2");
        assert_eq!(hit.output, vec![9]);
    }
}
