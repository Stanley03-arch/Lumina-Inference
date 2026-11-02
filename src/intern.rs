//! Simple string interning (dedupe repeated model / key strings).

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Interns strings so equal values share one `Arc<str>`.
#[derive(Debug, Clone, Default)]
pub struct StringInterner {
    set: Arc<Mutex<HashSet<Arc<str>>>>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&self, s: &str) -> Arc<str> {
        let mut set = self.set.lock().unwrap();
        if let Some(existing) = set.get(s) {
            return Arc::clone(existing);
        }
        let a: Arc<str> = Arc::from(s);
        set.insert(Arc::clone(&a));
        a
    }

    pub fn len(&self) -> usize {
        self.set.lock().unwrap().len()
    }

    pub fn clear(&self) {
        self.set.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedups() {
        let i = StringInterner::new();
        let a = i.intern("resnet");
        let b = i.intern("resnet");
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!(i.len(), 1);
    }
}
