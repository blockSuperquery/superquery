//! Substrate-facing type definitions — Rust port of `@subql/types`.
//!
//! Port status: **scaffold only**. Builds on [`subql_types_core`].

/// Crate name, used as a placeholder until real exports land.
pub const CRATE: &str = "subql-types";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depends_on_types_core() {
        assert_eq!(subql_types_core::CRATE, "subql-types-core");
        assert_eq!(CRATE, "subql-types");
    }
}
