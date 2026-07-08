//! SubQuery CLI — Rust port of `@subql/cli`.
//!
//! Port status: **scaffold only**. Largest TS package but least
//! performance-sensitive; intended to stay on TS the longest and be ported last.
//! Binary is named `subql` to match the published command.

fn main() {
    println!("subql (scaffold) — {}", subql_common::CRATE);
}
