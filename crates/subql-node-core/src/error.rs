//! Core error type shared across the engine seams.

use thiserror::Error;

/// Result alias used throughout `subql-node-core`.
pub type Result<T, E = CoreError> = std::result::Result<T, E>;

/// Errors surfaced by the indexing engine and its seams.
///
/// Kept intentionally small at M0; variants are added as subsystems land. The
/// `Other` variant carries `anyhow`-style context from chain/store implementors.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("block {height} not found")]
    BlockNotFound { height: u64 },

    #[error("data sources not found for height {height}")]
    DataSourcesNotFound { height: u64 },

    #[error("store error: {0}")]
    Store(String),

    #[error("chain/rpc error: {0}")]
    Chain(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
