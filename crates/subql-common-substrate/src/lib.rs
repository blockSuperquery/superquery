//! Substrate-specific common utilities — Rust port of `@subql/common-substrate`.
//!
//! Port status: **scaffold only**. Builds on [`subql_common`] and [`subql_types`].

/// Crate name, used as a placeholder until real exports land.
pub const CRATE: &str = "subql-common-substrate";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name() {
        assert_eq!(CRATE, "subql-common-substrate");
    }
}
