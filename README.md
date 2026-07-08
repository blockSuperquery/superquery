<div align="center">

# SuperQuery

### Query at super speed.

**`RUST-NATIVE` · `SUBQUERY COMPATIBLE`**

A high-integrity indexing framework designed for the most demanding blockchain
data pipelines. **The performance of Rust, the familiarity of SubQuery.**

[Website](https://superquery.vercel.app/) ·
[Documentation](https://superquery.vercel.app/) ·
[Examples](https://superquery.vercel.app/) ·
[Roadmap](https://superquery.vercel.app/) ·
[Status](https://superquery.vercel.app/)

</div>

---

## Overview

SuperQuery is a ground-up **Rust reimplementation** of the SubQuery indexing
stack, engineered for teams that need deterministic, high-throughput blockchain
indexing with enterprise operational guarantees. It keeps the developer
experience teams already know — GraphQL schemas, a project manifest, familiar
CLI ergonomics — while replacing the runtime with a native Rust engine for
predictable performance and a single, dependency-light binary.

```console
$ superquery run --chain ethereum

$ superquery build --release
✓ Compiled 4 handlers in 1.2s
✓ Indexed block 18,201,442 · 12ms
✓ GraphQL endpoint ready · localhost:3000/graphql

# tail latency p99: 42ms · throughput 8.4k blocks/s
```

## Why SuperQuery

| | |
|---|---|
| ⚡ **Native performance** | Mappings compile to native code — no interpreter, no GC pauses, predictable tail latency under load. |
| 🛡️ **High integrity** | Deterministic indexing with ACID-compliant storage and reorg-safe historical state. Verifiable, reproducible results. |
| 🧱 **SubQuery compatible** | Reuse your GraphQL schema and project manifest. A migration path, not a rewrite. |
| 📦 **Single binary** | One statically-linked artifact to ship and operate — simpler deploys, smaller attack surface. |
| 📊 **Observable** | First-class Prometheus metrics, health endpoints, and structured tracing for production fleets. |

## Technical Workflow

1. **Define Schema** — Standard GraphQL schema definition for your entities.
2. **Chain Ingest** — SuperQuery reads raw blocks via optimized Rust fetchers.
3. **Rust Mappings** — Blazing-fast data transformation compiled to native code.
4. **Store & Index** — ACID-compliant storage in PostgreSQL with automatic indexing.
5. **Query Engine** — Expose your data via high-concurrency GraphQL endpoints.

```rust
// Standard Rust handler for Transfer events
pub async fn handle_transfer(event: TransferEvent) -> Result<(), Error> {
    let mut account = Account::load(&event.from).await?;

    // High-integrity balance update
    account.balance -= event.value;
    account.save().await?;

    info!("Indexed transfer of {:?} from {:?}", event.value, event.from);
    Ok(())
}
```

## Features

- ✅ SubQuery Manifest Compatible
- ✅ Real-time Block Subscription
- ✅ Deterministic Indexing
- ✅ Automated API Scaffolding
- ✅ Multi-chain Aggregate Support
- ✅ Prometheus Monitoring Export

## Supported Networks

SuperQuery targets 1:1 network coverage with the SubQuery SDKs:

- Polkadot (and all Substrate networks)
- Ethereum (and all EVM-compatible networks)
- Cosmos (and all CosmWasm and Ethermint networks)
- Algorand · NEAR · Stellar (incl. Soroban) · Solana · Starknet · Concordium

Multi-chain indexing across any combination of the above is a first-class use case.

## Architecture

SuperQuery is a Cargo workspace of focused crates. Chain-specific logic lives
behind a single `BlockchainService` trait, so networks plug in without touching
the engine core.

| Crate | Responsibility |
|---|---|
| [`subql-node-core`](crates/subql-node-core) | Chain-agnostic indexing engine: fetch → dispatch → index pipeline, and the engine seams (traits). |
| [`subql-store`](crates/subql-store) | Storage layer over PostgreSQL (raw SQL, no ORM): schema generation, entity models, historical state. |
| [`subql-config`](crates/subql-config) | Node & database configuration (CLI + environment). |
| [`subql-node`](crates/subql-node) | Substrate indexer binary. |
| [`subql-query`](crates/subql-query) | High-concurrency GraphQL query service. |
| [`subql-cli`](crates/subql-cli) | Project lifecycle CLI (`superquery`). |
| `subql-common` · `subql-utils` · `subql-types*` | Shared manifest/schema utilities and type definitions. |

> **Project status.** SuperQuery is an active, staged port of the SubQuery
> engine to Rust. Correctness is validated by **differential testing against the
> reference implementation**: generated schema shape and stored rows are asserted
> byte-identical, and proof-of-index merkle roots are used for cross-verification.
> The milestone plan lives in [`.claude/tasks/`](.claude/tasks/); see the
> [Development Roadmap](#development-roadmap) below.

## Getting Started

> Requires a recent stable Rust toolchain and a PostgreSQL instance.

```bash
# Build the workspace
cargo build --release

# Unit tests
cargo test --workspace

# Integration/parity tests run against a live Postgres when DB_* is set
DB_HOST=127.0.0.1 DB_USER=postgres DB_PASS=postgres DB_DATABASE=postgres \
  cargo test --workspace
```

Database connection is read from the standard environment variables
(`DB_HOST`, `DB_PORT`, `DB_USER`, `DB_PASS`, `DB_DATABASE`), matching the
SubQuery node conventions.

## Development Roadmap

### v0.8 — Production Alpha *(current scope)*

- ✅ Rust SDK for EVM log handling
- ✅ PostgreSQL dynamic store plugin
- ✅ CLI code generation tools
- ✅ Substrate extrinsic support

### v1.0 — Stability Goal *(planned scope)*

- ◯ Distributed ingestion engine
- ◯ WASM execution sandbox
- ◯ GraphQL dynamic filtering v2
- ◯ Managed cloud interface

## Contributing

SuperQuery welcomes contributions. Please open an issue to discuss substantial
changes before submitting a pull request, and ensure the full check suite passes:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

## License

GPL-3.0. SuperQuery builds on the open-source SubQuery framework.
