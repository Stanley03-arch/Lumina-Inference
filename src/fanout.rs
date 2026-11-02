//! Fan-out: keep N subscribers and broadcast clones of events.

use std::sync::{Arc, Mutex};

/// Callback-style broadcaster (sync).
#[derive(Clone)]
pub struct Fanout<T> {
    listeners: Arc<Mutex<Vec<Box<dyn Fn(&T) + Send>>>>,
}

impl<T> Fanout<T> {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn subscribe<F>(&self, f: F)
    where
        F: Fn(&T) + Send + 'static,
    {
        self.listeners.lock().unwrap().push(Box::new(f));
    }

    pub fn listener_count(&self) -> usize {
        self.listeners.lock().unwrap().len()
    }

    pub fn publish(&self, event: &T) {
        let ls = self.listeners.lock().unwrap();
        for f in ls.iter() {
            f(event);
        }
    }

    pub fn clear(&self) {
        self.listeners.lock().unwrap().clear();
    }
}

impl<T> Default for Fanout<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn fans_out() {
        let f = Fanout::<u32>::new();
        let n = Arc::new(AtomicUsize::new(0));
        let n2 = n.clone();
        f.subscribe(move |_| {
            n2.fetch_add(1, Ordering::SeqCst);
        });
        f.publish(&1);
        f.publish(&2);
        assert_eq!(n.load(Ordering::SeqCst), 2);
    }
}
