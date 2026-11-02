//! Weighted round-robin selection (smooth WRR-style).

/// Entry with static weight.
#[derive(Debug, Clone)]
pub struct WeightedItem<T> {
    pub item: T,
    pub weight: u32,
    current: i64,
}

impl<T> WeightedItem<T> {
    pub fn new(item: T, weight: u32) -> Self {
        Self {
            item,
            weight: weight.max(1),
            current: 0,
        }
    }
}

/// Nginx-style smooth weighted round-robin.
#[derive(Debug, Clone, Default)]
pub struct WeightedRoundRobin<T> {
    items: Vec<WeightedItem<T>>,
}

impl<T> WeightedRoundRobin<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, item: T, weight: u32) {
        self.items.push(WeightedItem::new(item, weight));
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Pick next item index; returns `None` if empty.
    pub fn next_index(&mut self) -> Option<usize> {
        if self.items.is_empty() {
            return None;
        }
        let total: i64 = self.items.iter().map(|i| i.weight as i64).sum();
        let mut best = 0usize;
        for i in 0..self.items.len() {
            self.items[i].current += self.items[i].weight as i64;
            if self.items[i].current > self.items[best].current {
                best = i;
            }
        }
        self.items[best].current -= total;
        Some(best)
    }

    pub fn next(&mut self) -> Option<&T> {
        let i = self.next_index()?;
        Some(&self.items[i].item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn respects_weights() {
        let mut w = WeightedRoundRobin::new();
        w.push("a", 5);
        w.push("b", 1);
        let mut a = 0;
        let mut b = 0;
        for _ in 0..60 {
            match *w.next().unwrap() {
                "a" => a += 1,
                "b" => b += 1,
                _ => {}
            }
        }
        assert!(a > b * 3);
    }
}
