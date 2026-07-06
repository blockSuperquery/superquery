#!/usr/bin/env bash
# One-command bootstrap: scaffold a starter project if needed, then bring the stack up.
set -euo pipefail
cd "$(dirname "$0")"

PROJECT_DIR="${PROJECT_DIR:-./project}"
STARTER_REPO="https://github.com/subquery/subql-starter"
STARTER_PATH="Polkadot/Polkadot-starter"

if [ ! -d "$PROJECT_DIR" ]; then
  echo "==> No project found at $PROJECT_DIR — scaffolding the Polkadot starter"
  tmp="$(mktemp -d)"
  git clone --depth 1 "$STARTER_REPO" "$tmp/starter"
  cp -r "$tmp/starter/$STARTER_PATH" "$PROJECT_DIR"
  rm -rf "$tmp"
  # The starter's cast breaks against newer @polkadot/types pulled in by a fresh install
  sed -i 's/(amount as Balance)/(amount as unknown as Balance)/' \
    "$PROJECT_DIR/src/mappings/mappingHandlers.ts" 2>/dev/null || true
fi

# Mark the project as standalone so yarn doesn't treat it as part of this monorepo
[ -f "$PROJECT_DIR/yarn.lock" ] || touch "$PROJECT_DIR/yarn.lock"

if [ ! -d "$PROJECT_DIR/dist" ]; then
  echo "==> Building project (codegen + compile)"
  if command -v node >/dev/null 2>&1; then
    (cd "$PROJECT_DIR" && yarn install && yarn codegen && yarn build)
  else
    # No node on the host: build inside a container instead
    docker run --rm -v "$(cd "$PROJECT_DIR" && pwd)":/app -w /app node:lts \
      sh -c "yarn install && yarn codegen && yarn build"
  fi
fi

echo "==> Starting stack (postgres + indexer + graphql)"
docker compose up -d --build

echo ""
echo "GraphQL playground: http://localhost:${QUERY_PORT:-3000}"
echo "Follow indexer logs: docker compose -f $(pwd)/docker-compose.yml logs -f subquery-node"
