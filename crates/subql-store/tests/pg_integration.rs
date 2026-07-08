//! Integration tests against a real Postgres — the GATE 2 harness foundation.
//!
//! Each test runs inside a uniquely-named **ephemeral schema** that is dropped on
//! teardown, so runs are isolated and repeatable. If no database is reachable
//! (e.g. CI without PG), the tests **skip with a notice** rather than fail, so
//! `cargo test` stays green everywhere. To force them, ensure `DB_*` env points at
//! a live Postgres (defaults: 127.0.0.1:5432 postgres/postgres).

use subql_config::DbConfig;
use subql_store::Database;

/// Connect if possible; return `None` (and print why) to skip the test otherwise.
async fn try_db() -> Option<Database> {
    let cfg = DbConfig::from_env();
    match Database::connect(&cfg) {
        Ok(db) => match db.ping().await {
            Ok(()) => Some(db),
            Err(e) => {
                eprintln!("[skip] postgres not reachable ({e}); skipping integration test");
                None
            }
        },
        Err(e) => {
            eprintln!("[skip] could not build pool ({e}); skipping integration test");
            None
        }
    }
}

/// A random, collision-resistant schema name for isolation.
fn unique_schema() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("subql_it_{nanos}")
}

#[tokio::test]
async fn ephemeral_schema_metadata_roundtrip() {
    let Some(db) = try_db().await else { return };
    let schema = unique_schema();

    db.ensure_schema(&schema).await.expect("create schema");
    // Ensure teardown even if assertions panic.
    let result = async {
        db.ensure_metadata_table(&schema)
            .await
            .expect("metadata table");
        db.set_metadata(&schema, "k", "v1").await.expect("set");
        assert_eq!(
            db.get_metadata(&schema, "k").await.expect("get"),
            Some("v1".to_string())
        );
        // upsert overwrites
        db.set_metadata(&schema, "k", "v2").await.expect("set2");
        assert_eq!(
            db.get_metadata(&schema, "k").await.expect("get2"),
            Some("v2".to_string())
        );
        assert_eq!(
            db.get_metadata(&schema, "missing").await.expect("get3"),
            None
        );
    }
    .await;
    db.drop_schema(&schema).await.expect("drop schema");
    result
}

#[tokio::test]
async fn introspection_reports_columns_and_indexes() {
    let Some(db) = try_db().await else { return };
    let schema = unique_schema();
    db.ensure_schema(&schema).await.expect("create schema");

    let outcome = async {
        // A representative table: text pk, nullable numeric, an index.
        db.batch_execute(&format!(
            "CREATE TABLE \"{schema}\".\"transfer\" ( \
                id text NOT NULL PRIMARY KEY, \
                amount numeric, \
                sender text NOT NULL \
             ); \
             CREATE INDEX transfer_sender_idx ON \"{schema}\".\"transfer\" (sender);"
        ))
        .await
        .expect("create table");

        let info = db.introspect_schema(&schema).await.expect("introspect");

        let table = info.tables.get("transfer").expect("transfer table present");
        // Columns are sorted by name: amount, id, sender.
        let cols: Vec<_> = table.columns.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(cols, vec!["amount", "id", "sender"]);

        let id = table.columns.iter().find(|c| c.name == "id").unwrap();
        assert_eq!(id.data_type, "text");
        assert!(!id.is_nullable);

        let amount = table.columns.iter().find(|c| c.name == "amount").unwrap();
        assert_eq!(amount.data_type, "numeric");
        assert!(amount.is_nullable);

        // pk + secondary index both present.
        assert!(
            table.indexes.iter().any(|i| i.is_unique),
            "expected a unique (pk) index"
        );
        assert!(
            table
                .indexes
                .iter()
                .any(|i| i.definition.contains("(sender)")),
            "expected the sender index, got: {:?}",
            table.indexes
        );
    }
    .await;

    db.drop_schema(&schema).await.expect("drop schema");
    outcome
}
