//! Node configuration — Rust port of `@subql/node-core` `NodeConfig` + `yargs`.
//!
//! Field names, defaults, and env-var names are kept **1:1** with the TS impl
//! (`configure/NodeConfig.ts` `IConfig`/`DEFAULT_CONFIG`, and `db/db.module.ts`
//! for DB connection env vars) so behaviour matches exactly. See the port plan
//! `.claude/tasks/node-core-rust-port.md`.
//!
//! Port status: M0 subset — the full `IConfig` field set with correct defaults
//! is modelled; CLI wiring covers the flags needed through GATE 1 and grows as
//! milestones land.

mod db;
mod node;

pub use db::DbConfig;
pub use node::{HistoricalMode, NodeConfig};
