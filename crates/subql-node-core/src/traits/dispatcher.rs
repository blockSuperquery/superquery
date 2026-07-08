//! Block dispatcher + indexer manager seams — ports of `IBlockDispatcher`
//! (`blockDispatcher/base-block-dispatcher.ts`) and `IIndexerManager`
//! (`indexer/types.ts`).

use async_trait::async_trait;

use crate::error::Result;
use crate::types::{Header, IBlock};

/// Outcome of processing one block — port of TS `ProcessBlockResponse`.
#[derive(Debug, Clone, Default)]
pub struct ProcessBlockResponse {
    /// A dynamic datasource was created while indexing this block.
    pub dynamic_ds_created: bool,
    /// A reorg was detected; if set, the engine rewinds to this header.
    pub reindex_block_header: Option<Header>,
}

/// An item queued for indexing: either a bare height (fetch later) or a
/// already-fetched block (e.g. supplied by the dictionary). Mirrors the TS
/// `(IBlock<B> | number)[]` enqueue signature.
pub enum EnqueuedBlock<B> {
    Height(u64),
    Block(B),
}

/// Drives fetching + processing of blocks — port of `IBlockDispatcher<B>`.
#[async_trait]
pub trait BlockDispatcher: Send + Sync {
    /// Concrete fetched-block type carried through the queue.
    type FetchedBlock: IBlock;

    async fn init(&self) -> Result<()>;

    /// Enqueue blocks and advance the buffered height.
    async fn enqueue_blocks(
        &self,
        blocks: Vec<EnqueuedBlock<Self::FetchedBlock>>,
        latest_buffer_height: u64,
    ) -> Result<()>;

    fn queue_size(&self) -> usize;
    fn free_size(&self) -> usize;
    fn latest_buffered_height(&self) -> u64;
    fn batch_size(&self) -> u32;

    fn set_latest_processed_height(&self, height: u64);
    /// Drop all queued blocks (used on reorg / new dynamic datasource).
    fn flush_queue(&self, height: u64);
}

/// Indexes a single block against its datasources — port of `IIndexerManager`.
#[async_trait]
pub trait IndexerManager: Send + Sync {
    type Block: IBlock;
    type DataSource;

    async fn index_block(
        &self,
        block: Self::Block,
        data_sources: &[Self::DataSource],
    ) -> Result<ProcessBlockResponse>;
}
