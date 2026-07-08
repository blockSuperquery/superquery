//! Substrate SubQuery indexer — Rust port of `@subql/node`.
//!
//! Port status: **scaffold only**. Wires the Substrate blockchain service into
//! [`subql_node_core`].

fn main() {
    println!(
        "subql-node (scaffold) — engine: {}, chain: {}",
        subql_node_core::CRATE,
        subql_common_substrate::CRATE,
    );
}
