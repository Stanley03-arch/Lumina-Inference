//! Model artifact integrity (size + SHA-256).

use crate::error::{LuminaError, Result};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Expected metadata for a model file.
#[derive(Debug, Clone)]
pub struct ModelDigest {
    pub path: std::path::PathBuf,
    /// Optional expected byte length.
    pub size: Option<u64>,
    /// Optional lowercase hex SHA-256.
    pub sha256_hex: Option<String>,
}

impl ModelDigest {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            path: path.into(),
            size: None,
            sha256_hex: None,
        }
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_sha256_hex(mut self, hex: impl Into<String>) -> Self {
        self.sha256_hex = Some(hex.into().to_lowercase());
        self
    }
}

/// Hash a file with SHA-256; returns lowercase hex.
pub fn sha256_file(path: impl AsRef<Path>) -> Result<String> {
    let f = File::open(path.as_ref())
        .map_err(|e| LuminaError::Config(format!("open model: {e}")))?;
    let mut reader = BufReader::new(f);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| LuminaError::Config(format!("read model: {e}")))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex_encode(&hasher.finalize()))
}

/// Verify size and/or SHA-256 against [`ModelDigest`].
pub fn verify_model(digest: &ModelDigest) -> Result<()> {
    let meta = std::fs::metadata(&digest.path)
        .map_err(|e| LuminaError::Config(format!("stat {}: {e}", digest.path.display())))?;
    let len = meta.len();
    if let Some(expected) = digest.size {
        if len != expected {
            return Err(LuminaError::Config(format!(
                "size mismatch for {}: got {len}, expected {expected}",
                digest.path.display()
            )));
        }
    }
    if let Some(ref expected) = digest.sha256_hex {
        let got = sha256_file(&digest.path)?;
        if got != expected.to_lowercase() {
            return Err(LuminaError::Config(format!(
                "sha256 mismatch for {}: got {got}, expected {expected}",
                digest.path.display()
            )));
        }
    }
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}

// --- Minimal SHA-256 (public domain style, no external crate) ---

struct Sha256 {
    state: [u32; 8],
    buffer: [u8; 64],
    buflen: usize,
    bitlen: u64,
}

impl Sha256 {
    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
                0x1f83d9ab, 0x5be0cd19,
            ],
            buffer: [0; 64],
            buflen: 0,
            bitlen: 0,
        }
    }

    fn update(&mut self, data: &[u8]) {
        for &b in data {
            self.buffer[self.buflen] = b;
            self.buflen += 1;
            if self.buflen == 64 {
                self.transform();
                self.bitlen += 512;
                self.buflen = 0;
            }
        }
    }

    fn finalize(mut self) -> [u8; 32] {
        let i = self.buflen;
        self.bitlen += (self.buflen as u64) * 8;
        self.buffer[i] = 0x80;
        if i >= 56 {
            for j in (i + 1)..64 {
                self.buffer[j] = 0;
            }
            self.transform();
            for j in 0..56 {
                self.buffer[j] = 0;
            }
        } else {
            for j in (i + 1)..56 {
                self.buffer[j] = 0;
            }
        }
        let bits = self.bitlen;
        for k in 0..8 {
            self.buffer[63 - k] = (bits >> (8 * k)) as u8;
        }
        self.transform();
        let mut out = [0u8; 32];
        for (i, &s) in self.state.iter().enumerate() {
            out[i * 4] = (s >> 24) as u8;
            out[i * 4 + 1] = (s >> 16) as u8;
            out[i * 4 + 2] = (s >> 8) as u8;
            out[i * 4 + 3] = s as u8;
        }
        out
    }

    fn transform(&mut self) {
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];
        let mut m = [0u32; 64];
        for i in 0..16 {
            let j = i * 4;
            m[i] = ((self.buffer[j] as u32) << 24)
                | ((self.buffer[j + 1] as u32) << 16)
                | ((self.buffer[j + 2] as u32) << 8)
                | (self.buffer[j + 3] as u32);
        }
        for i in 16..64 {
            let s0 = m[i - 15].rotate_right(7) ^ m[i - 15].rotate_right(18) ^ (m[i - 15] >> 3);
            let s1 = m[i - 2].rotate_right(17) ^ m[i - 2].rotate_right(19) ^ (m[i - 2] >> 10);
            m[i] = m[i - 16]
                .wrapping_add(s0)
                .wrapping_add(m[i - 7])
                .wrapping_add(s1);
        }
        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];
        let mut f = self.state[5];
        let mut g = self.state[6];
        let mut h = self.state[7];
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(m[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn sha256_empty_known() {
        // SHA-256("") =
        // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let dir = std::env::temp_dir();
        let path = dir.join("lumina_sha_empty");
        File::create(&path).unwrap().write_all(b"").unwrap();
        let h = sha256_file(&path).unwrap();
        assert_eq!(
            h,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn verify_size() {
        let dir = std::env::temp_dir();
        let path = dir.join("lumina_size_chk");
        File::create(&path).unwrap().write_all(b"abc").unwrap();
        let d = ModelDigest::new(&path).with_size(3);
        assert!(verify_model(&d).is_ok());
        let d2 = ModelDigest::new(&path).with_size(9);
        assert!(verify_model(&d2).is_err());
        let _ = std::fs::remove_file(&path);
    }
}
