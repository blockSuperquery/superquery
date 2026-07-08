#!/usr/bin/env node
/*
 * Ground-truth ROW-DATA fixture generator (GATE 2, data half).
 *
 * Builds the schema via the real TS path (same as gen-schema-fixture.cjs),
 * inserts a fixed set of input entities through the Sequelize model, then dumps
 * the stored rows canonicalized (col::text / encode(bytea,'hex')) — identical
 * rendering to the Rust `PlainModel::dump_canonical`. The Rust parity test upserts
 * the same input and asserts identical output.
 *
 * NOTE: the chosen input values store identically with or without Sequelize's
 * get/set column hooks (BigInt passed as string, arrays as JSON, bytea NULL), so
 * this fixture is authentic without needing node-core's hook logic. Bytes/Date
 * value-encoding parity gets its own hook-driven fixture in a later slice.
 *
 * Usage: node scripts/gen-data-fixture.cjs <schema.graphql> <input.json> <out.json> [pgSchema]
 */
const fs = require('fs');
const path = require('path');
const {getAllEntitiesRelations, getTypeByScalarName, buildSchemaFromString} = require('@subql/utils');
const {Sequelize} = require('@subql/x-sequelize');

const [, , schemaPath, inputPath, outPath, pgSchemaArg] = process.argv;
if (!schemaPath || !inputPath || !outPath) {
  console.error('usage: gen-data-fixture.cjs <schema.graphql> <input.json> <out.json> [pgSchema]');
  process.exit(2);
}
const pgSchema = pgSchemaArg || 'subql_ts_data';

function sequelizeType(field) {
  if (field.isEnum) throw new Error('enum not covered by this fixture slice');
  if (field.isArray || field.jsonInterface) return getTypeByScalarName('Json').sequelizeType;
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

function underscore(name) {
  return name
    .replace(/([A-Z\d]+)([A-Z][a-z])/g, '$1_$2')
    .replace(/([a-z\d])([A-Z])/g, '$1_$2')
    .replace(/-/g, '_')
    .toLowerCase();
}

async function main() {
  const raw = fs.readFileSync(path.resolve(schemaPath), 'utf8');
  const input = JSON.parse(fs.readFileSync(path.resolve(inputPath), 'utf8'));
  const {models} = getAllEntitiesRelations(buildSchemaFromString(raw));
  const model = models[0];

  const sequelize = new Sequelize(
    process.env.DB_DATABASE || 'postgres',
    process.env.DB_USER || 'postgres',
    process.env.DB_PASS || 'postgres',
    {host: process.env.DB_HOST || '127.0.0.1', port: Number(process.env.DB_PORT || 5432), dialect: 'postgres', logging: false}
  );
  await sequelize.authenticate();
  await sequelize.dropSchema(pgSchema, {}).catch(() => {});
  await sequelize.createSchema(pgSchema, {}).catch(() => {});

  const seqModel = sequelize.define(model.name, modelAttributes(model), {
    underscored: true,
    freezeTableName: false,
    timestamps: false,
    schema: pgSchema,
  });
  await sequelize.sync();

  const allKeys = Object.keys(seqModel.getAttributes());
  await seqModel.bulkCreate(input, {updateOnDuplicate: allKeys});

  // Canonical dump: col::text (bytea → hex), sorted columns, ordered by id.
  const cols = model.fields.map((f) => underscore(f.name)).sort();
  const projection = model.fields
    .map((f) => {
      const c = underscore(f.name);
      return f.type === 'Bytes' && !f.isArray
        ? `encode("${c}", 'hex') AS "${c}"`
        : `"${c}"::text AS "${c}"`;
    })
    .join(', ');
  const table = seqModel.getTableName().tableName ?? seqModel.getTableName();
  const [rows] = await sequelize.query(
    `SELECT ${projection} FROM "${pgSchema}"."${table}" ORDER BY id`
  );

  // Normalize into sorted-key objects to match the Rust BTreeMap dump.
  const normalized = rows.map((r) => {
    const o = {};
    for (const c of cols) o[c] = r[c] === undefined ? null : r[c];
    return o;
  });

  fs.writeFileSync(path.resolve(outPath), JSON.stringify(normalized, null, 2) + '\n');
  console.log(JSON.stringify(normalized, null, 2));

  await sequelize.dropSchema(pgSchema, {}).catch(() => {});
  await sequelize.close();
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
