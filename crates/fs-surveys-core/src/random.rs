//! The only source of randomness: the operating system, through `getrandom`.
//!
//! Pattern: Facade — insertion positions, Fisher-Yates shuffles and tokens all come from here.

use crate::error::{Error, Result};

/// Fill `buf` from the system.
pub fn fill(buf: &mut [u8]) -> Result<()> {
    getrandom::fill(buf).map_err(|e| Error::Random(e.to_string()))
}

/// A uniform number in `0..n` (`n > 0`), by rejection so there is no bias.
pub fn below(n: usize) -> Result<usize> {
    debug_assert!(n > 0);
    let n = n as u64;
    let zone = u64::MAX - (u64::MAX % n);
    loop {
        let mut b = [0u8; 8];
        fill(&mut b)?;
        let x = u64::from_le_bytes(b);
        if x < zone {
            return Ok(usize::try_from(x % n).unwrap_or(0));
        }
    }
}

/// Fisher-Yates shuffle.
pub fn shuffle<T>(items: &mut [T]) -> Result<()> {
    for i in (1..items.len()).rev() {
        items.swap(i, below(i + 1)?);
    }
    Ok(())
}

/// Insert at a random position (so the order says nothing about the order of arrival).
pub fn insert_random<T>(items: &mut Vec<T>, item: T) -> Result<()> {
    let at = below(items.len() + 1)?;
    items.insert(at, item);
    Ok(())
}

/// 256 random bits as 64 hex characters.
pub fn token() -> Result<String> {
    let mut b = [0u8; 32];
    fill(&mut b)?;
    Ok(hex(&b))
}

/// Lower-case hex.
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}
