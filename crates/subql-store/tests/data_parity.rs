//! GATE 2 (data half): the Rust `PlainModel` must store rows byte-identically to
//! the TS node, and its CRUD/query semantics must match.
//!
//! `data.expected.json` is ground truth from `scripts/gen-data-fixture.cjs`
//! (real TS insert via Sequelize, canonical `::text` dump). This test upserts the
//! same `data.input.json` through the Rust `PlainModel` and asserts the canonical
//! dump matches, then exercises upsert-overwrite, delete, get, and get_by_fields.
//!
//! Skips if Postgres is unreachable.

use std::collections::BTreeMap;

use serde_json::Value;
use subql_config::DbConfig;
use subql_store::{ddl, parse_entities, Database, OrderDir, PlainModel, QueryOptions};

type Row = BTreeMap<String, Option<String>>;

async fn try_db() -> Option<Database> {
    let cfg = DbConfig::from_env();
    match Database::connect(&cfg) {
        Ok(db) => db.ping().await.ok().map(|_| db),
        Err(_) => None,
    }
}

fn unique_schema() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("subql_data_{n}")
}

async fn setup(db: &Database, schema: &str) -> PlainModel {
    let sdl = include_str!("fixtures/schema.graphql");
    let models = parse_entities(sdl).expect("parse");
    db.ensure_schema(schema).await.expect("schema");
    for stmt in ddl::create_tables(&models, schema).expect("ddl") {
        db.batch_execute(&stmt).await.expect("apply ddl");
    }
    PlainModel::new(schema, &models[0])
}

fn input_rows() -> Vec<Value> {
    serde_json::from_str(include_str!("fixtures/data.input.json")).expect("input")
}

#[tokio::test]
async fn row_data_matches_ts_ground_truth() {
    let Some(db) = try_db().await else { return };
    let schema = unique_schema();
    let model = setup(&db, &schema).await;

    let outcome = async {
        model.upsert(&db, &input_rows()).await.expect("upsert");

        let got: Vec<Row> = model.dump_canonical(&db).await.expect("dump");
        let expected: Vec<Row> =
            serde_json::from_str(include_str!("fixtures/data.expected.json")).expect("expected");

        assert_eq!(got, expected, "Rust row dump differs from TS ground truth");
    }
    .await;

    db.drop_schema(&schema).await.expect("drop");
    outcome
}

#[tokio::test]
async fn upsert_is_last_write_wins() {
    let Some(db) = try_db().await else { return };
    let schema = unique_schema();
    let model = setup(&db, &schema).await;

    let outcome = async {
        let base = &input_rows()[0];
        model
            .upsert(&db, std::slice::from_ref(base))
            .await
            .expect("insert");

        // Re-upsert same id with a changed field.
        let mut updated = base.clone();
        updated["sender"] = Value::String("zoe".into());
        model
            .upsert(&db, std::slice::from_ref(&updated))
            .await
            .expect("update");

        let row = model.get(&db, "0x01").await.expect("get").expect("present");
        assert_eq!(row.get("sender").unwrap().as_deref(), Some("zoe"));

        // Still a single row.
        assert_eq!(model.dump_canonical(&db).await.expect("dump").len(), 1);
    }
    .await;
    db.drop_schema(&schema).await.expect("drop");
    outcome
}

#[tokio::test]
async fn remove_and_query_semantics() {
    let Some(db) = try_db().await else { return };
    let schema = unique_schema();
    let model = setup(&db, &schema).await;

    let outcome = async {
        model.upsert(&db, &input_rows()).await.expect("upsert");

        // Delete one; the other remains.
        model
            .remove(&db, &["0x01".to_string()])
            .await
            .expect("remove");
        assert!(model.get(&db, "0x01").await.expect("get").is_none());
        assert!(model.get(&db, "0x02").await.expect("get").is_some());

        // Re-insert and test ordered/paginated query by an equality filter.
        model.upsert(&db, &input_rows()).await.expect("re-upsert");
        let opts = QueryOptions {
            order_by: "blockHeight".into(),
            order_direction: OrderDir::Desc,
            limit: 1,
            offset: 0,
        };
        let rows = model
            .get_by_fields(&db, &[("success".into(), Value::Bool(true))], &opts)
            .await
            .expect("query");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("id").unwrap().as_deref(), Some("0x01"));
    }
    .await;
    db.drop_schema(&schema).await.expect("drop");
    outcome
}
