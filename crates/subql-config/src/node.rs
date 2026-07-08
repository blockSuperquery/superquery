//! `NodeConfig` — 1:1 port of `configure/NodeConfig.ts` `IConfig` + `DEFAULT_CONFIG`.

use clap::Parser;
use serde::{Deserialize, Serialize};

/// Historical indexing mode. Mirrors TS `HistoricalMode = 'height' | 'timestamp' | false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HistoricalMode {
    /// `'height'` — track entity versions by block height (TS default).
    #[default]
    Height,
    /// `'timestamp'` — track by block timestamp (used for multichain).
    Timestamp,
    /// `false` — historical disabled.
    #[serde(other)]
    Disabled,
}

/// The full node configuration surface.
///
/// Defaults reproduce `DEFAULT_CONFIG` (NodeConfig.ts lines 73-99) exactly.
/// Fields the TS marks optional are `Option<_>`; the rest carry TS defaults.
#[derive(Debug, Clone, Parser)]
#[command(name = "subql-node", about = "SubQuery indexer node (Rust port)")]
pub struct NodeConfig {
    /// Path/locator of the project to run.
    #[arg(long, env = "SUBQL_SUBQUERY")]
    pub subquery: String,

    /// Number of blocks fetched/processed per batch.
    #[arg(long, default_value_t = 100)]
    pub batch_size: u32,

    /// Task timeout in seconds.
    #[arg(long, default_value_t = 900)]
    pub timeout: u64,

    /// Expected block time in milliseconds.
    #[arg(long, default_value_t = 6000)]
    pub block_time: u64,

    /// Prefer range queries in the fetch service.
    #[arg(long, default_value_t = false)]
    pub prefer_range: bool,

    /// GraphQL query limit for store getByField.
    #[arg(long, default_value_t = 100)]
    pub query_limit: u32,

    /// Historical indexing mode. Not yet a CLI flag — derived/defaulted for now,
    /// resolved from the project + `--historical` in a later milestone.
    #[clap(skip = HistoricalMode::Height)]
    pub historical: HistoricalMode,

    /// Enable proof-of-index.
    #[arg(long, default_value_t = false)]
    pub proof_of_index: bool,

    /// Index unfinalized (best) blocks and handle reorgs.
    #[arg(long)]
    pub unfinalized_blocks: Option<bool>,

    /// Number of worker threads. `None` = single-process.
    #[arg(long)]
    pub workers: Option<u32>,

    /// Multi-chain indexing mode.
    #[arg(long, default_value_t = false)]
    pub multi_chain: bool,

    /// RPC endpoint(s) for the network.
    #[arg(long = "network-endpoint", env = "SUBQL_NETWORK_ENDPOINT")]
    pub network_endpoints: Vec<String>,

    /// Store write-behind cache: flush threshold (records).
    #[arg(long, default_value_t = 1000)]
    pub store_cache_threshold: u32,

    /// Store get-cache max size.
    #[arg(long, default_value_t = 500)]
    pub store_get_cache_size: u32,

    /// Store flush interval (blocks).
    #[arg(long, default_value_t = 5)]
    pub store_flush_interval: u32,

    /// Log level (`trace|debug|info|warn|error|silent`).
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Root directory of the project (resolved at runtime when absent).
    #[arg(long)]
    pub root: Option<String>,
}

impl NodeConfig {
    /// Construct with all TS defaults for a given project locator — the
    /// equivalent of `new NodeConfig({subquery})`. Handy for tests.
    pub fn with_defaults(subquery: impl Into<String>) -> Self {
        Self {
            subquery: subquery.into(),
            batch_size: 100,
            timeout: 900,
            block_time: 6000,
            prefer_range: false,
            query_limit: 100,
            historical: HistoricalMode::Height,
            proof_of_index: false,
            unfinalized_blocks: None,
            workers: None,
            multi_chain: false,
            network_endpoints: Vec::new(),
            store_cache_threshold: 1000,
            store_get_cache_size: 500,
            store_flush_interval: 5,
            log_level: "info".to_string(),
            root: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_ts_default_config() {
        // Assert every default matches DEFAULT_CONFIG in NodeConfig.ts.
        let c = NodeConfig::with_defaults("./project");
        assert_eq!(c.batch_size, 100);
        assert_eq!(c.timeout, 900);
        assert_eq!(c.block_time, 6000);
        assert_eq!(c.query_limit, 100);
        assert!(!c.proof_of_index);
        assert_eq!(c.historical, HistoricalMode::Height);
        assert!(!c.multi_chain);
        assert_eq!(c.store_cache_threshold, 1000);
        assert_eq!(c.store_get_cache_size, 500);
        assert_eq!(c.store_flush_interval, 5);
        assert_eq!(c.log_level, "info");
        assert_eq!(c.workers, None);
    }

    #[test]
    fn parses_from_cli_args() {
        let c = NodeConfig::try_parse_from([
            "subql-node",
            "--subquery",
            "./my-project",
            "--batch-size",
            "50",
            "--proof-of-index",
            "--workers",
            "4",
        ])
        .expect("should parse");
        assert_eq!(c.subquery, "./my-project");
        assert_eq!(c.batch_size, 50);
        assert!(c.proof_of_index);
        assert_eq!(c.workers, Some(4));
    }
}
