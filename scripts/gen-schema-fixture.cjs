#!/usr/bin/env node
/*
 * Ground-truth fixture generator for the Rust store port (GATE 2).
 *
 * Runs the *real* TS path — @subql/utils `getAllEntitiesRelations` + the exact
 * column-option logic from node-core `utils/graphql.ts#getColumnOption`, defined
 * on @subql/x-sequelize with the same options as `store.service.ts#defineModel`,
 * then `sync()` — against a live Postgres. It then introspects the created schema
 * from information_schema and writes it as the authoritative fixture JSON.
 *
 * The Rust DDL generator must reproduce this introspected shape exactly.
 *
 * Usage: node scripts/gen-schema-fixture.cjs <schema.graphql> <out.json> [pgSchema]
 *   DB connection from DB_HOST/DB_PORT/DB_USER/DB_PASS/DB_DATABASE.
 */
const fs = require('fs');
const path = require('path');
const {getAllEntitiesRelations, getTypeByScalarName, buildSchemaFromString} = require('@subql/utils');
const {Sequelize} = require('@subql/x-sequelize');

const [, , schemaPath, outPath, pgSchemaArg] = process.argv;
if (!schemaPath || !outPath) {
  console.error('usage: gen-schema-fixture.cjs <schema.graphql> <out.json> [pgSchema]');
  process.exit(2);
}
const pgSchema = pgSchemaArg || 'subql_ts_fixture';

// Faithful port target: node-core utils/graphql.ts#getColumnOption (non-enum path).
function sequelizeType(field) {
  if (field.isEnum) throw new Error('enum fields not covered by this fixture slice');
  if (field.isArray || field.jsonInterface) {
    return getTypeByScalarName('Json').sequelizeType; // arrays & json → JSONB
  }
  const t = getTypeByScalarName(field.type);
  if (!t) throw new Error(`unknown scalar type ${field.type}`);
  return t.sequelizeType;
}

function modelAttributes(model) {
  const attrs = {};
  for (const field of model.fields) {
    attrs[field.name] = {
      type: sequelizeType(field),
      allowNull: field.nullable,
      primaryKey: field.type === 'ID',
    };
  }
  return attrs;
}

async function main() {
  const raw = fs.readFileSync(path.resolve(schemaPath), 'utf8');
  const {models} = getAllEntitiesRelations(buildSchemaFromString(raw));

  const sequelize = new Sequelize(
    process.env.DB_DATABASE || 'postgres',
    process.env.DB_USER || 'postgres',
    process.env.DB_PASS || 'postgres',
    {
      host: process.env.DB_HOST || '127.0.0.1',
      port: Number(process.env.DB_PORT || 5432),
      dialect: 'postgres',
      logging: false,
    }
  );

  await sequelize.authenticate();
  await sequelize.createSchema(pgSchema, {}).catch(() => {});

  // Mirror store.service.ts#defineModel options for the non-historical case.
  for (const model of models) {
    sequelize.define(model.name, modelAttributes(model), {
      underscored: true,
      freezeTableName: false,
      timestamps: false,
      schema: pgSchema,
    });
  }
  await sequelize.sync();

  // Introspect the created schema (same queries as subql-store::introspect).
  const [cols] = await sequelize.query(
    `SELECT table_name, column_name, COALESCE(domain_name, udt_name) AS data_type,
            is_nullable, column_default
     FROM information_schema.columns WHERE table_schema = :s
     ORDER BY table_name, column_name`,
    {replacements: {s: pgSchema}}
  );

  const tables = {};
  for (const c of cols) {
    (tables[c.table_name] ??= {columns: []}).columns.push({
      name: c.column_name,
      data_type: c.data_type,
      is_nullable: c.is_nullable === 'YES',
      default: c.column_default,
    });
  }

  const fixture = {source: path.basename(schemaPath), pgSchema, tables};
  fs.writeFileSync(path.resolve(outPath), JSON.stringify(fixture, null, 2) + '\n');
  console.log(JSON.stringify(fixture, null, 2));

  await sequelize.dropSchema(pgSchema, {}).catch(() => {});
  await sequelize.close();
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
