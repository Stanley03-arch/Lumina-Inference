//! Engine lifecycle phases.

use serde::{Deserialize, Serialize};

/// High-level process phase for readiness / admin status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Phase {
    #[default]
    Starting,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl Phase {
    pub fn as_str(self) -> &'static str {
        match self {
            Phase::Starting => "starting",
            Phase::Warming => "warming",
            Phase::Ready => "ready",
            Phase::Draining => "draining",
            Phase::Stopped => "stopped",
        }
    }

    pub fn accepts_traffic(self) -> bool {
        matches!(self, Phase::Ready)
    }

    /// Valid transition check (best-effort linear lifecycle).
    pub fn can_transition_to(self, next: Phase) -> bool {
        use Phase::*;
        matches!(
            (self, next),
            (Starting, Warming)
                | (Starting, Ready)
                | (Warming, Ready)
                | (Ready, Draining)
                | (Draining, Stopped)
                | (Starting, Stopped)
                | (Warming, Stopped)
                | (Ready, Stopped)
                | (_, _) if self == next
        )
    }
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitions() {
        assert!(Phase::Starting.can_transition_to(Phase::Ready));
        assert!(!Phase::Stopped.can_transition_to(Phase::Ready));
        assert!(Phase::Ready.accepts_traffic());
    }
}
