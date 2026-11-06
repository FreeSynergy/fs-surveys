//! Hashing of tokens.
//!
//! Pattern: Facade — the one place that turns a token into what the box keeps: `sha256(token)` in hex.

use sha2::{Digest, Sha256};

use crate::random::hex;

/// What the box keeps of a token.
pub fn hash_token(token: &str) -> String {
    hex(&Sha256::digest(token.as_bytes()))
}
