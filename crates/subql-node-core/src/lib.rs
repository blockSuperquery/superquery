//! Core chain-agnostic indexing engine — Rust port of `@subql/node-core`.
//!
//! Port status: **scaffold only**. This is the crux of the migration: block
//! fetch, reorg handling, the store/DB layer, pipeline orchestration, and the
//! mapping-execution sandbox. The mapping-execution strategy (embed a JS runtime
//! vs. IPC to a Node worker vs. Rust/WASM mappings) is still an open decision and
//! governs the shape of this crate.

/// Crate name, used as a placeholder until real exports land.
pub const CRATE: &str = "subql-node-core";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name() {
        assert_eq!(CRATE, "subql-node-core");
    }
}
