//! General utility functions — Rust port of `@subql/utils`.
//!
//! Port status: **scaffold only**. Hashing, merkle, and formatting helpers land
//! here and must match the TS output byte-for-byte (golden-vector tests).

/// Crate name, used as a placeholder until real exports land.
pub const CRATE: &str = "subql-utils";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name() {
        assert_eq!(CRATE, "subql-utils");
    }
}
