//! Append-only JSON Lines writer for events / audit export.

use crate::error::{LuminaError, Result};
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Writes one JSON object per line to a file.
#[derive(Debug)]
pub struct JsonlWriter {
    path: PathBuf,
}

impl JsonlWriter {
    pub fn open(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append<T: Serialize>(&self, value: &T) -> Result<()> {
        let line = serde_json::to_string(value)
            .map_err(|e| LuminaError::Internal(format!("jsonl encode: {e}")))?;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| LuminaError::Internal(format!("jsonl open: {e}")))?;
        writeln!(f, "{line}").map_err(|e| LuminaError::Internal(format!("jsonl write: {e}")))?;
        Ok(())
    }

    pub fn append_str(&self, line: &str) -> Result<()> {
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| LuminaError::Internal(format!("jsonl open: {e}")))?;
        writeln!(f, "{line}").map_err(|e| LuminaError::Internal(format!("jsonl write: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize)]
    struct Ev {
        id: u32,
    }

    #[test]
    fn writes_lines() {
        let path = std::env::temp_dir().join("lumina_jsonl_test.jsonl");
        let _ = std::fs::remove_file(&path);
        let w = JsonlWriter::open(&path);
        w.append(&Ev { id: 1 }).unwrap();
        w.append(&Ev { id: 2 }).unwrap();
        let data = std::fs::read_to_string(&path).unwrap();
        assert_eq!(data.lines().count(), 2);
        let _ = std::fs::remove_file(&path);
    }
}
