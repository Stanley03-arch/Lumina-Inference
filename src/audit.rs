//! In-memory ring buffer of recent request audit events.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// One audited request outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub ts_ms: u64,
    pub request_id: String,
    pub model: String,
    pub worker: String,
    pub latency_ms: u64,
    pub ok: bool,
    pub error: Option<String>,
}

/// Fixed-capacity ring of recent events (newest at the back).
#[derive(Debug, Clone)]
pub struct AuditLog {
    events: VecDeque<AuditEvent>,
    capacity: usize,
}

impl AuditLog {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity.min(10_000)),
            capacity: capacity.max(1),
        }
    }

    pub fn record_ok(
        &mut self,
        request_id: impl Into<String>,
        model: impl Into<String>,
        worker: impl Into<String>,
        latency_ms: u64,
    ) {
        self.push(AuditEvent {
            ts_ms: now_ms(),
            request_id: request_id.into(),
            model: model.into(),
            worker: worker.into(),
            latency_ms,
            ok: true,
            error: None,
        });
    }

    pub fn record_err(
        &mut self,
        request_id: impl Into<String>,
        model: impl Into<String>,
        worker: impl Into<String>,
        latency_ms: u64,
        error: impl Into<String>,
    ) {
        self.push(AuditEvent {
            ts_ms: now_ms(),
            request_id: request_id.into(),
            model: model.into(),
            worker: worker.into(),
            latency_ms,
            ok: false,
            error: Some(error.into()),
        });
    }

    fn push(&mut self, ev: AuditEvent) {
        if self.events.len() >= self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(ev);
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Newest first.
    pub fn recent(&self, n: usize) -> Vec<AuditEvent> {
        self.events.iter().rev().take(n).cloned().collect()
    }

    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(&self.events).unwrap_or_else(|_| "[]".into())
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_evicts() {
        let mut log = AuditLog::new(2);
        log.record_ok("1", "m", "w", 1);
        log.record_ok("2", "m", "w", 2);
        log.record_ok("3", "m", "w", 3);
        assert_eq!(log.len(), 2);
        let r = log.recent(10);
        assert_eq!(r[0].request_id, "3");
        assert_eq!(r[1].request_id, "2");
    }
}
