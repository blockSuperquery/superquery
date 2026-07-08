//! `ProjectService` — port of the TS `IProjectService<DS>` (`indexer/types.ts`).
//! Owns datasource resolution and reindex orchestration.

use async_trait::async_trait;

use crate::error::Result;
use crate::types::{BlockHeightMap, BypassBlocks, Header};

#[async_trait]
pub trait ProjectService: Send + Sync {
    type DataSource: Clone + Send + Sync;

    /// Height the project starts indexing from.
    fn start_height(&self) -> u64;

    /// Optional offset applied to block heights.
    fn block_offset(&self) -> Option<u64>;

    /// Heights/ranges to skip.
    fn bypass_blocks(&self) -> &BypassBlocks;

    /// All datasources — used everywhere except while indexing a block.
    fn get_all_data_sources(&self) -> Vec<Self::DataSource>;

    /// Datasources active for a height. Async because workers may need to sync
    /// dynamic datasources first.
    async fn get_data_sources(&self, block_height: Option<u64>) -> Result<Vec<Self::DataSource>>;

    /// The start block computed from datasource definitions.
    fn get_start_block_from_data_sources(&self) -> u64;

    /// Height → active datasources map (drives fetch ranges & bypass gaps).
    fn get_data_sources_map(&self) -> &BlockHeightMap<Vec<Self::DataSource>>;

    /// Whether any datasource applies strictly after `height` (controls whether
    /// the store can finalize/compact past this height).
    fn has_data_sources_after_height(&self, height: u64) -> bool;

    /// Roll back indexed state to `last_correct_header` (reorg / admin rewind).
    async fn reindex(&self, last_correct_header: Header) -> Result<()>;
}
