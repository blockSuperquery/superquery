//! GraphQL query service — Rust port of `@subql/query`.
//!
//! Port status: **scaffold only**. Recommended first real service to port: it
//! speaks HTTP/GraphQL over Postgres with no mapping sandbox, so it can run
//! side-by-side with the TS service and cut over per-deployment.

fn main() {
    println!("subql-query (scaffold) — {}", subql_common::CRATE);
}
