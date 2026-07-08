//! Postgres connection primitive over `deadpool-postgres` (mirrors the TS `pg`
//! Pool in `db/db.module.ts`). M0 scope: connect, ensure a schema exists, and a
//! minimal `_metadata` upsert/get so GATE 1 can prove real-DB round-trips.

use deadpool_postgres::{Config as PoolConfig, Pool, Runtime};
use subql_config::DbConfig;
use tokio_postgres::NoTls;

use crate::error::StoreError;

/// A pooled Postgres connection, scoped to a single project schema.
pub struct Database {
    pool: Pool,
}

impl Database {
    /// Build a connection pool from [`DbConfig`]. Does not open a connection until
    /// first use (deadpool is lazy).
    pub fn connect(cfg: &DbConfig) -> Result<Self, StoreError> {
        let mut pool_cfg = PoolConfig::new();
        pool_cfg.host = Some(cfg.host.clone());
        pool_cfg.port = Some(cfg.port);
        pool_cfg.user = Some(cfg.username.clone());
        pool_cfg.password = Some(cfg.password.clone());
        pool_cfg.dbname = Some(cfg.database.clone());

        let pool = pool_cfg
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| StoreError::Pool(e.to_string()))?;
        Ok(Self { pool })
    }

    /// Round-trip check: `SELECT 1`. Proves the pool can reach the server.
    pub async fn ping(&self) -> Result<(), StoreError> {
        let client = self.conn().await?;
        client.query_one("SELECT 1", &[]).await?;
        Ok(())
    }

    /// Borrow a pooled client (internal helper).
    async fn conn(&self) -> Result<deadpool_postgres::Object, StoreError> {
        self.pool
            .get()
            .await
            .map_err(|e| StoreError::Pool(e.to_string()))
    }

    /// `DROP SCHEMA ... CASCADE` — used by ephemeral test schemas for teardown.
    pub async fn drop_schema(&self, schema: &str) -> Result<(), StoreError> {
        validate_ident(schema)?;
        let client = self.conn().await?;
        client
            .batch_execute(&format!("DROP SCHEMA IF EXISTS \"{schema}\" CASCADE"))
            .await?;
        Ok(())
    }

    /// Run raw SQL (test/setup helper; not for user input).
    pub async fn batch_execute(&self, sql: &str) -> Result<(), StoreError> {
        let client = self.conn().await?;
        client.batch_execute(sql).await?;
        Ok(())
    }

    /// Execute a parameterized statement, returning the affected row count.
    pub async fn execute(
        &self,
        sql: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, StoreError> {
        let client = self.conn().await?;
        Ok(client.execute(sql, params).await?)
    }

    /// Run a parameterized query, returning the rows.
    pub async fn query(
        &self,
        sql: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, StoreError> {
        let client = self.conn().await?;
        Ok(client.query(sql, params).await?)
    }

    /// Introspect `schema` into a canonical, comparison-ready [`SchemaInfo`].
    pub async fn introspect_schema(
        &self,
        schema: &str,
    ) -> Result<crate::introspect::SchemaInfo, StoreError> {
        let client = self.conn().await?;
        crate::introspect::introspect(&client, schema).await
    }

    /// `CREATE SCHEMA IF NOT EXISTS`. Schema name is validated to be a safe
    /// identifier (it cannot be a bound parameter in DDL).
    pub async fn ensure_schema(&self, schema: &str) -> Result<(), StoreError> {
        validate_ident(schema)?;
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StoreError::Pool(e.to_string()))?;
        client
            .batch_execute(&format!("CREATE SCHEMA IF NOT EXISTS \"{schema}\""))
            .await?;
        Ok(())
    }

    /// Ensure a minimal key/value `_metadata` table exists in `schema`.
    /// (The real metadata model with its full column set lands in M1.)
    pub async fn ensure_metadata_table(&self, schema: &str) -> Result<(), StoreError> {
        validate_ident(schema)?;
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StoreError::Pool(e.to_string()))?;
        client
            .batch_execute(&format!(
                "CREATE TABLE IF NOT EXISTS \"{schema}\".\"_metadata\" \
                 (key text PRIMARY KEY, value text NOT NULL)"
            ))
            .await?;
        Ok(())
    }

    /// Upsert a metadata key/value.
    pub async fn set_metadata(
        &self,
        schema: &str,
        key: &str,
        value: &str,
    ) -> Result<(), StoreError> {
        validate_ident(schema)?;
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StoreError::Pool(e.to_string()))?;
        client
            .execute(
                &format!(
                    "INSERT INTO \"{schema}\".\"_metadata\" (key, value) VALUES ($1, $2) \
                     ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value"
                ),
                &[&key, &value],
            )
            .await?;
        Ok(())
    }

    /// Read a metadata value by key.
    pub async fn get_metadata(
        &self,
        schema: &str,
        key: &str,
    ) -> Result<Option<String>, StoreError> {
        validate_ident(schema)?;
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StoreError::Pool(e.to_string()))?;
        let row = client
            .query_opt(
                &format!("SELECT value FROM \"{schema}\".\"_metadata\" WHERE key = $1"),
                &[&key],
            )
            .await?;
        Ok(row.map(|r| r.get::<_, String>(0)))
    }
}

/// Postgres identifiers are interpolated into DDL (they can't be bound
/// parameters), so restrict them to a safe charset to prevent injection.
fn validate_ident(ident: &str) -> Result<(), StoreError> {
    let ok = !ident.is_empty()
        && ident.len() <= 63
        && ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if ok {
        Ok(())
    } else {
        Err(StoreError::InvalidSchema(ident.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsafe_identifiers() {
        assert!(validate_ident("app").is_ok());
        assert!(validate_ident("app_1").is_ok());
        assert!(validate_ident("bad-name").is_err());
        assert!(validate_ident("drop\";--").is_err());
        assert!(validate_ident("").is_err());
    }
}
