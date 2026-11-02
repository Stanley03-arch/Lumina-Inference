//! Fixed-capacity ring buffer (overwrite oldest).

/// Circular buffer of `T` with fixed capacity.
#[derive(Debug, Clone)]
pub struct RingBuf<T> {
    buf: Vec<Option<T>>,
    head: usize,
    len: usize,
}

impl<T> RingBuf<T> {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        let mut buf = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buf.push(None);
        }
        Self {
            buf,
            head: 0,
            len: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == self.buf.len()
    }

    pub fn push(&mut self, item: T) {
        let idx = (self.head + self.len) % self.buf.len();
        if self.len == self.buf.len() {
            // overwrite oldest
            self.buf[self.head] = Some(item);
            self.head = (self.head + 1) % self.buf.len();
        } else {
            self.buf[idx] = Some(item);
            self.len += 1;
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let item = self.buf[self.head].take();
        self.head = (self.head + 1) % self.buf.len();
        self.len -= 1;
        item
    }

    pub fn clear(&mut self) {
        for slot in &mut self.buf {
            *slot = None;
        }
        self.head = 0;
        self.len = 0;
    }

    /// Oldest → newest snapshot.
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut out = Vec::with_capacity(self.len);
        for i in 0..self.len {
            let idx = (self.head + i) % self.buf.len();
            if let Some(ref v) = self.buf[idx] {
                out.push(v.clone());
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overwrites() {
        let mut r = RingBuf::new(2);
        r.push(1);
        r.push(2);
        r.push(3);
        assert_eq!(r.to_vec(), vec![2, 3]);
    }
}
