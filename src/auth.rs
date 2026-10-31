//! Static API-token authentication (Bearer / raw token).

use crate::error::{LuminaError, Result};
use std::collections::HashSet;

/// Accepts a fixed set of API tokens (constant-time-ish compare via full scan).
#[derive(Debug, Clone, Default)]
pub struct TokenAuth {
    tokens: HashSet<String>,
    /// When true, empty token set means auth disabled (allow all).
    pub allow_if_empty: bool,
}

impl TokenAuth {
    pub fn new(tokens: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            tokens: tokens.into_iter().map(|t| t.into()).collect(),
            allow_if_empty: true,
        }
    }

    /// Load from `LUMINA_API_TOKENS` (comma-separated).
    pub fn from_env() -> Self {
        let mut auth = Self::default();
        auth.allow_if_empty = true;
        if let Ok(raw) = std::env::var("LUMINA_API_TOKENS") {
            for part in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                auth.tokens.insert(part.to_string());
            }
        }
        auth
    }

    pub fn insert(&mut self, token: impl Into<String>) {
        self.tokens.insert(token.into());
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Validate a raw token or `Bearer <token>` header value.
    pub fn check(&self, credential: Option<&str>) -> Result<()> {
        if self.tokens.is_empty() && self.allow_if_empty {
            return Ok(());
        }
        let Some(raw) = credential.map(str::trim).filter(|s| !s.is_empty()) else {
            return Err(LuminaError::Unauthorized("missing token".into()));
        };
        let token = raw
            .strip_prefix("Bearer ")
            .or_else(|| raw.strip_prefix("bearer "))
            .unwrap_or(raw)
            .trim();
        if self.tokens.iter().any(|t| t == token) {
            Ok(())
        } else {
            Err(LuminaError::Unauthorized("invalid token".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_bearer() {
        let a = TokenAuth::new(["secret"]);
        assert!(a.check(Some("Bearer secret")).is_ok());
        assert!(a.check(Some("secret")).is_ok());
        assert!(a.check(Some("Bearer wrong")).is_err());
        assert!(a.check(None).is_err());
    }

    #[test]
    fn empty_allows() {
        let a = TokenAuth::new(Vec::<String>::new());
        assert!(a.check(None).is_ok());
    }
}
