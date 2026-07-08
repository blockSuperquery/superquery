//! Shared utilities and types — Rust port of `@subql/common`.
//!
//! Port status: **scaffold only**. Project manifest parsing/validation lands here.

/// Crate name, used as a placeholder until real exports land.
pub const CRATE: &str = "subql-common";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name() {
        assert_eq!(CRATE, "subql-common");
    }
}
