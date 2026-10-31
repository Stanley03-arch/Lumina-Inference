//! Simple LRU cache of model id → local artifact path (or handle key).

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone)]
struct Entry {
    path: PathBuf,
    last: Instant,
}

/// Maps model ids to filesystem paths with LRU eviction.
#[derive(Debug, Clone)]
pub struct ModelCache {
    map: HashMap<String, Entry>,
    max: usize,
}

impl ModelCache {
    pub fn new(max: usize) -> Self {
        Self {
            map: HashMap::new(),
            max: max.max(1),
        }
    }

    pub fn insert(&mut self, model_id: impl Into<String>, path: impl Into<PathBuf>) {
        let id = model_id.into();
        if self.map.len() >= self.max && !self.map.contains_key(&id) {
            if let Some(oldest) = self
                .map
                .iter()
                .min_by_key(|(_, e)| e.last)
                .map(|(k, _)| k.clone())
            {
                self.map.remove(&oldest);
            }
        }
        self.map.insert(
            id,
            Entry {
                path: path.into(),
                last: Instant::now(),
            },
        );
    }

    pub fn get(&mut self, model_id: &str) -> Option<PathBuf> {
        if let Some(e) = self.map.get_mut(model_id) {
            e.last = Instant::now();
            return Some(e.path.clone());
        }
        None
    }

    pub fn remove(&mut self, model_id: &str) -> Option<PathBuf> {
        self.map.remove(model_id).map(|e| e.path)
    }

    pub fn contains(&self, model_id: &str) -> bool {
        self.map.contains_key(model_id)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}

impl Default for ModelCache {
    fn default() -> Self {
        Self::new(32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lru_evict() {
        let mut c = ModelCache::new(2);
        c.insert("a", "/a.onnx");
        c.insert("b", "/b.onnx");
        let _ = c.get("a"); // touch a
        c.insert("c", "/c.onnx"); // should drop b
        assert!(c.contains("a"));
        assert!(c.contains("c"));
        assert!(!c.contains("b"));
    }
}
