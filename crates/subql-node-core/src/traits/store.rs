//! Store seams — ports of the user-facing `Store` (`indexer/store/store.ts`), the
//! model-level `IModel` (`storeModelProvider/model/model.ts`), and the
//! `IStoreModelProvider` (`storeModelProvider/types.ts`).
//!
//! Entities are represented dynamically as JSON objects (`Entity`) because the
//! schema is generated at runtime from the project's GraphQL — there is no static
//! Rust type per entity. `subql-store` (M1) provides the concrete implementations
//! over raw SQL.

use async_trait::async_trait;
use serde_json::Value;

use crate::error::Result;

/// A dynamic entity: a JSON object that always carries a string `id`.
pub type Entity = Value;

/// Sort direction for `getByField(s)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection {
    Asc,
    Desc,
}

/// Query options — port of types-core `GetOptions`.
#[derive(Debug, Clone)]
pub struct GetOptions {
    pub offset: u32,
    pub limit: u32,
    pub order_by: String,
    pub order_direction: OrderDirection,
}

/// Comparison operators for `FieldExpression`. The string forms must match
/// `operatorsMap` in `storeModelProvider/model/utils.ts` (reconciled in M1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldOperator {
    Eq,
    Ne,
    In,
    NotIn,
    Lt,
    Lte,
    Gt,
    Gte,
}

/// A single `[field, operator, value]` filter — port of `FieldsExpression`.
#[derive(Debug, Clone)]
pub struct FieldExpression {
    pub field: String,
    pub operator: FieldOperator,
    pub value: Value,
}

/// Model-level data access — port of `IModel<T>`.
///
/// `block_height` is the historical unit (height or timestamp) at which the write
/// applies. A concrete impl is either the write-behind `CachedModel` or the
/// direct `PlainModel`.
#[async_trait]
pub trait Model: Send + Sync {
    async fn get(&self, id: &str) -> Result<Option<Entity>>;
    async fn get_by_fields(
        &self,
        filters: &[FieldExpression],
        options: &GetOptions,
    ) -> Result<Vec<Entity>>;

    async fn set(&self, id: &str, data: Entity, block_height: u64) -> Result<()>;
    async fn bulk_create(&self, data: Vec<Entity>, block_height: u64) -> Result<()>;
    async fn bulk_update(
        &self,
        data: Vec<Entity>,
        block_height: u64,
        fields: Option<Vec<String>>,
    ) -> Result<()>;
    async fn bulk_remove(&self, ids: Vec<String>, block_height: u64) -> Result<()>;
}

/// The API injected into user mappings — port of the user-facing `Store`.
/// Index-existence checks and query-limit enforcement live in the concrete impl.
#[async_trait]
pub trait Store: Send + Sync {
    async fn get(&self, entity: &str, id: &str) -> Result<Option<Entity>>;
    async fn get_by_field(
        &self,
        entity: &str,
        field: &str,
        value: Value,
        options: &GetOptions,
    ) -> Result<Vec<Entity>>;
    async fn get_by_fields(
        &self,
        entity: &str,
        filters: &[FieldExpression],
        options: &GetOptions,
    ) -> Result<Vec<Entity>>;
    async fn get_one_by_field(
        &self,
        entity: &str,
        field: &str,
        value: Value,
    ) -> Result<Option<Entity>>;

    async fn set(&self, entity: &str, id: &str, data: Entity) -> Result<()>;
    async fn bulk_create(&self, entity: &str, data: Vec<Entity>) -> Result<()>;
    async fn bulk_update(
        &self,
        entity: &str,
        data: Vec<Entity>,
        fields: Option<Vec<String>>,
    ) -> Result<()>;
    async fn remove(&self, entity: &str, id: &str) -> Result<()>;
    async fn bulk_remove(&self, entity: &str, ids: Vec<String>) -> Result<()>;
}

/// Provides models + metadata/poi and orchestrates cache flushing — port of
/// `IStoreModelProvider`. Two impls in M1: cached (write-behind) and plain.
#[async_trait]
pub trait StoreModelProvider: Send + Sync {
    /// Flush pending changes for `height`. `data_sources_completed` signals no
    /// datasources remain after this height (enables compaction).
    async fn apply_pending_changes(
        &self,
        height: u64,
        data_sources_completed: bool,
    ) -> Result<()>;
}
