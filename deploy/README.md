# Simplified deployment (no k8s)

Everything runs on a single machine with Docker Compose: three containers, one command.

```
./start.sh
```

That scaffolds a Polkadot starter project into `./project` (if you don't have one), builds it, builds the node + query images from this repo's source, and starts the stack. The GraphQL playground comes up at http://localhost:3000.

## What runs

| Service | Built from | Role |
| --- | --- | --- |
| `postgres` | [pg-Dockerfile](../packages/node/docker/pg-Dockerfile) | Postgres 16 + `btree_gist` extension (required for historical/time-travel queries) |
| `subquery-node` | [packages/node](../packages/node/Dockerfile) | The indexer. Reads blocks from the chain RPC defined in your project manifest, runs your mapping handlers in a sandbox, writes entities into the `app` Postgres schema |
| `graphql-engine` | [packages/query](../packages/query/Dockerfile) | PostGraphile-based GraphQL server. Introspects the `app` schema and serves the API your users hit |

The data flow: chain RPC → `subquery-node` → Postgres → `graphql-engine` → users.

## Using your own project

Point the stack at any built SubQuery project (a directory with `project.yaml`/`project.ts`, `schema.graphql`, and a compiled `dist/`):

```
PROJECT_DIR=/path/to/your-project ./start.sh
```

Or drop it in `deploy/project/` and run `docker compose up -d`.

## Knobs

Set via environment or a `.env` file next to the compose file:

- `DB_PASS` — postgres password (default `postgres`; change it before exposing anything)
- `QUERY_PORT` — public GraphQL port (default `3000`)
- `WORKERS`, `BATCH_SIZE` — indexer throughput (defaults `2` / `10`; raise on bigger machines)
- `PROJECT_DIR` — project directory mounted into the indexer (default `./project`)

To use the published images instead of building from source, swap the `build:` blocks for the commented `image:` lines in [docker-compose.yml](docker-compose.yml).

## Going public

Put a reverse proxy (Caddy/nginx) in front of `QUERY_PORT` for TLS and rate limiting, remove `--playground` from the `graphql-engine` command if you don't want the interactive playground exposed, and don't publish the postgres port (delete its `ports:` mapping — the containers talk over the compose network).
