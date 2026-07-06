#!/bin/sh
# Build and pack a workspace package for the Docker production stage.
# Run from the repo root after `yarn install`. Compiles the target package
# plus its workspace dependencies (topologically), then packs it. Yarn 4's
# pack rewrites workspace:~ ranges to the concrete local versions.
set -e

name=$(cd "$1" && node -p "require('./package.json').name")
yarn workspaces foreach -tR --from "$name" run build

cd "$1"
yarn pack --out app.tgz
