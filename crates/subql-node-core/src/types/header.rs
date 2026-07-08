//! Block header + block wrapper — port of `indexer/types.ts` `Header` / `IBlock`.

use chrono::{DateTime, Utc};

/// A minimal, chain-agnostic block header.
///
/// Mirrors the TS `Header` type. `timestamp` is `Option` here even though the TS
/// type declares it required: several call sites (`updateStoreMetadata`,
/// `StoreService.rewind`) guard it as possibly-absent, and the docs on
/// `IBlockchainService.getBlockTimestamp` note some chains (e.g. Shiden) don't
/// expose one. Modelling it as `Option` matches runtime behaviour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub block_height: u64,
    pub block_hash: String,
    pub parent_hash: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

/// A fetched block: its header plus the chain-specific block payload `B`.
///
/// Port of the TS `IBlock<B>` interface (`getHeader()` + `block`). Implemented as
/// a trait so chain crates can wrap their native block types without copying.
pub trait IBlock: Send + Sync {
    /// The chain-specific block payload type.
    type Inner;

    fn header(&self) -> &Header;
    fn block(&self) -> &Self::Inner;
    fn into_inner(self) -> Self::Inner;
}

/// The straightforward `(Header, B)` implementation of [`IBlock`]. Chain crates
/// may use this directly or provide their own zero-copy wrapper.
#[derive(Debug, Clone)]
pub struct GenericBlock<B> {
    pub header: Header,
    pub inner: B,
}

impl<B> GenericBlock<B> {
    pub fn new(header: Header, inner: B) -> Self {
        Self { header, inner }
    }
}

impl<B: Send + Sync> IBlock for GenericBlock<B> {
    type Inner = B;

    fn header(&self) -> &Header {
        &self.header
    }
    fn block(&self) -> &B {
        &self.inner
    }
    fn into_inner(self) -> B {
        self.inner
    }
}
