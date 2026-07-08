//! `BlockchainService` — the chain-abstraction seam, port of the TS
//! `IBlockchainService` interface (`blockchain.service.ts`).
//!
//! This is the single interface each network crate (`subql-node` = Substrate,
//! plus EVM/Cosmos/… later) implements. The engine is generic over it. Methods
//! that only matter to deferred subsystems (workers, dynamic/custom datasources)
//! are intentionally omitted here and added when those milestones land; each is
//! noted below so the seam stays honest about scope.

use async_trait::async_trait;

use crate::error::Result;
use crate::types::{Header, IBlock};

/// Chain-specific services required by the engine.
///
/// Associated types replace the ~9 TS generic parameters:
/// - `DataSource` ↔ `DS`
/// - `Block`      ↔ the chain's full block payload `B`
/// - `SafeApi`    ↔ `SafeAPI` (a height-pinned API handed to mappings)
#[async_trait]
pub trait BlockchainService: Send + Sync {
    type DataSource: Send + Sync;
    type Block: Send + Sync;
    type SafeApi: Send + Sync;
    /// Concrete fetched-block type (implements [`IBlock`] over `Block`).
    type FetchedBlock: IBlock<Inner = Self::Block>;

    /// Semver of the running node (`packageVersion`).
    fn package_version(&self) -> &str;

    /// Handler kind string for block handlers (`blockHandlerKind`).
    fn block_handler_kind(&self) -> &str;

    // --- Fetch service: chain-height tracking ---

    /// The finalized (or probabilistically-finalized) head.
    async fn get_finalized_header(&self) -> Result<Header>;

    /// The latest/best (possibly unfinalized) height.
    async fn get_best_height(&self) -> Result<u64>;

    /// Approximate block interval in milliseconds.
    async fn get_chain_interval_ms(&self) -> Result<u64>;

    // --- Header lookups ---

    async fn get_header_for_height(&self, height: u64) -> Result<Header>;
    async fn get_header_for_hash(&self, hash: &str) -> Result<Header>;

    /// Block timestamp; some chains require an extra request (e.g. Shiden).
    async fn get_block_timestamp(&self, height: u64) -> Result<Option<chrono::DateTime<chrono::Utc>>>;

    // --- Block dispatcher ---

    /// Fetch the given block numbers.
    async fn fetch_blocks(&self, block_nums: &[u64]) -> Result<Vec<Self::FetchedBlock>>;

    /// Size of a block, used to compute a rolling median for batch scaling.
    fn get_block_size(&self, block: &Self::FetchedBlock) -> usize;

    // --- Indexer manager ---

    /// A height-pinned API so state queries reflect the block being indexed.
    async fn get_safe_api(&self, block: &Self::Block) -> Result<Self::SafeApi>;

    // --- Datasource discrimination ---

    fn is_custom_ds(&self, ds: &Self::DataSource) -> bool;
    fn is_runtime_ds(&self, ds: &Self::DataSource) -> bool;

    // Deferred to later milestones (kept out of the seam until implemented):
    //   fetch_block_worker(..)        — workers (M5)
    //   update_dynamic_ds(..)         — dynamic datasources (M5)
    //   on_project_change(..)         — project upgrades (M5)
}
