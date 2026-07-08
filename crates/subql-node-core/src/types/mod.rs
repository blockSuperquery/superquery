//! Core value types shared across the engine — port of `indexer/types.ts` and
//! `utils/blockHeightMap.ts`.

mod block_height_map;
mod header;
mod operation;

pub use block_height_map::{BlockHeightMap, EntryNotFoundError, GetRange};
pub use header::{GenericBlock, Header, IBlock};
pub use operation::{OperationData, OperationEntity, OperationType};

/// Blocks to skip while indexing. Each element is either a single height or an
/// inclusive `"start-end"` range string. Port of TS `BypassBlocks`.
pub type BypassBlocks = Vec<BypassRange>;

/// One entry of [`BypassBlocks`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BypassRange {
    Single(u64),
    Range(u64, u64),
}
