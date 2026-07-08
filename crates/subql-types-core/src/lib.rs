//! Core type definitions — Rust port of `@subql/types-core`.
//!
//! Port status: **scaffold only**. Add the real type surface as it is migrated
//! from `packages/types-core`.

/// Crate name, used as a placeholder until real exports land.
pub const CRATE: &str = "subql-types-core";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name() {
        assert_eq!(CRATE, "subql-types-core");
    }
}
