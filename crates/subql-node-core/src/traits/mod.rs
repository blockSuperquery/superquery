//! Engine seams — the trait boundaries that chain crates and the store implement.
//! These replace the TS NestJS-injected interfaces (`IBlockchainService`,
//! `IProjectService`, `IStoreModelProvider`, `IBlockDispatcher`, …).

mod blockchain;
mod dispatcher;
mod project;
mod store;

pub use blockchain::BlockchainService;
pub use dispatcher::{BlockDispatcher, EnqueuedBlock, IndexerManager, ProcessBlockResponse};
pub use project::ProjectService;
pub use store::{
    Entity, FieldExpression, FieldOperator, GetOptions, Model, OrderDirection, Store,
    StoreModelProvider,
};
