//! Core chain-agnostic indexing engine — Rust port of `@subql/node-core`.
//!
//! Port status: **M0** — establishing seams and foundations. Currently ships a
//! minimal JSON-RPC height probe ([`rpc`]) used by the `subql-smoke` binary to
//! prove real chain connectivity at GATE 1. The indexing pipeline (fetch,
//! dispatch, indexer manager, sandbox) lands across M2–M4.
//!
//! The mapping-execution strategy (embedded `deno_core` vs Rust/WASM) is resolved
//! at M4 — see `.claude/tasks/node-core-rust-port.md` §3.

pub mod rpc;

/// Crate name, retained as a stable placeholder export.
pub const CRATE: &str = "subql-node-core";
