//! Store operation types — port of `indexer/types.ts` `OperationType` /
//! `OperationEntity`. These feed the POI merkle tree (see M5 POI milestone).

use serde_json::Value;

/// A store mutation kind. String values (`"Set"`/`"Remove"`) are load-bearing:
/// they are hashed into the POI leaf bytes, so they must match the TS exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Set,
    Remove,
}

impl OperationType {
    /// The exact string used by the TS impl when serialising to POI bytes.
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationType::Set => "Set",
            OperationType::Remove => "Remove",
        }
    }
}

/// The payload of a store operation. For `Remove` this is the id string; for
/// `Set` it is the full entity object.
#[derive(Debug, Clone)]
pub enum OperationData {
    /// Entity id (used by `Remove`).
    Id(String),
    /// Full entity (used by `Set`).
    Entity(Value),
}

/// A recorded store operation. Mirrors TS `OperationEntity`.
#[derive(Debug, Clone)]
pub struct OperationEntity {
    pub operation: OperationType,
    pub entity_type: String,
    pub data: OperationData,
}
