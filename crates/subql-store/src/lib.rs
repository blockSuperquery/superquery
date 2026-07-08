//! Store layer — Rust port of `@subql/node-core` store (raw SQL, no ORM).
//!
//! Port status: **M0** — connection primitive only (Postgres pool + schema +
//! a minimal `_metadata` read/write to prove real-DB connectivity at GATE 1).
//! The write-behind cache (`CachedModel`), dynamic entity DDL, and historical
//! `_block_range` handling arrive in M1. See `.claude/tasks/node-core-rust-port.md`.

mod db;
pub mod ddl;
mod error;
pub mod introspect;
pub mod model;
pub mod naming;
pub mod schema;

pub use db::Database;
pub use error::StoreError;
pub use introspect::{ColumnInfo, IndexInfo, SchemaInfo, TableInfo};
pub use model::{CanonicalRow, OrderDir, PlainModel, QueryOptions};
pub use schema::{parse_entities, EntityField, EntityModel};
