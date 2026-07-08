use thiserror::Error;

/// Errors surfaced by the store layer.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("postgres error: {0}")]
    Postgres(#[from] tokio_postgres::Error),

    #[error("connection pool error: {0}")]
    Pool(String),

    #[error("invalid schema name: {0}")]
    InvalidSchema(String),

    #[error("failed to parse GraphQL schema: {0}")]
    Schema(String),

    #[error("unsupported GraphQL type: {0}")]
    UnsupportedType(String),
}
