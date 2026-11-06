//! The ballot box: where anonymous answers and unspent token hashes live.
//!
//! Pattern: Facade (declarations) — `model` is the pure data, `io` reads and writes it in the ballot store.

mod io;
mod model;
