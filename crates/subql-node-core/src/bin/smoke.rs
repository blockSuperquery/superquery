//! GATE 1 smoke test — proves the Rust foundations can talk to the real world
//! before any indexing logic is built on them.
//!
//! Checks:
//!   1. Config parses from CLI + env.
//!   2. Real Postgres: connect, create a schema, write and read back a
//!      `_metadata` row.
//!   3. Real chain RPC: fetch the current height from a live endpoint.
//!
//! Usage:
//!   subql-smoke --endpoint <RPC_URL> --family substrate|evm --schema smoke_test
//!   (DB connection is read from DB_HOST/DB_PORT/DB_USER/DB_PASS/DB_DATABASE)

use anyhow::{Context, Result};
use clap::Parser;
use subql_config::DbConfig;
use subql_node_core::rpc::{ChainFamily, JsonRpcClient};
use subql_store::Database;

#[derive(Parser)]
#[command(name = "subql-smoke", about = "GATE 1: real RPC + real DB connectivity check")]
struct Args {
    /// Chain RPC endpoint (HTTP JSON-RPC).
    #[arg(long, env = "SUBQL_NETWORK_ENDPOINT")]
    endpoint: String,

    /// Chain family for the height probe.
    #[arg(long, default_value = "substrate")]
    family: String,

    /// Postgres schema to create/use for the round-trip check.
    #[arg(long, default_value = "subql_smoke")]
    schema: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    load_dotenv();
    tracing_subscriber::fmt().with_env_filter("info").init();
    let args = Args::parse();

    let family = match args.family.to_lowercase().as_str() {
        "substrate" => ChainFamily::Substrate,
        "evm" => ChainFamily::Evm,
        other => anyhow::bail!("unknown --family '{other}', expected 'substrate' or 'evm'"),
    };

    println!("== GATE 1: foundations live ==\n");

    // --- Check 2: real Postgres round-trip ---
    let db_cfg = DbConfig::from_env();
    println!(
        "[db]  connecting to postgres {}:{}/{} (user={})",
        db_cfg.host, db_cfg.port, db_cfg.database, db_cfg.username
    );
    let db = Database::connect(&db_cfg).context("build pg pool")?;
    db.ping().await.context("postgres ping (SELECT 1)")?;
    db.ensure_schema(&args.schema).await.context("create schema")?;
    db.ensure_metadata_table(&args.schema).await.context("create _metadata")?;

    let stamp = run_stamp();
    db.set_metadata(&args.schema, "smoke_lastRun", &stamp).await.context("write metadata")?;
    let read_back = db
        .get_metadata(&args.schema, "smoke_lastRun")
        .await
        .context("read metadata")?;
    anyhow::ensure!(
        read_back.as_deref() == Some(stamp.as_str()),
        "metadata round-trip mismatch: wrote {stamp:?}, read {read_back:?}"
    );
    println!("[db]  ✓ schema \"{}\" ready, _metadata round-trip OK (smoke_lastRun={stamp})\n", args.schema);

    // --- Check 3: real chain RPC ---
    println!("[rpc] probing {} ({:?})", args.endpoint, family);
    let client = JsonRpcClient::new(&args.endpoint);
    let height = client
        .latest_height(family)
        .await
        .context("fetch latest height from RPC")?;
    println!("[rpc] ✓ latest height = {height}\n");

    println!("== GATE 1 PASSED ==");
    Ok(())
}

/// Best-effort load of a local `.env` (so `DB_*` / `SUBQL_*` can live in a file).
/// Existing process env always wins; missing file is fine.
fn load_dotenv() {
    let path = std::path::Path::new(".env");
    let Ok(contents) = std::fs::read_to_string(path) else {
        return;
    };
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let k = k.trim();
            let v = v.trim().trim_matches('"');
            if !k.is_empty() && std::env::var_os(k).is_none() {
                std::env::set_var(k, v);
            }
        }
    }
}

/// A unique-per-run marker value for the metadata round-trip check.
fn run_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    format!("unixtime:{secs}")
}
