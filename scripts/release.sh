#!/usr/bin/env bash
# scripts/release.sh <major|minor|patch|X.Y.Z>
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

case "$1" in
major | minor | patch | alpha | beta | rc | release) cargo set-version --workspace --bump "$1" ;;
*) cargo set-version --workspace "$1" ;;
esac

V=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
npm --prefix npm version "$V" --no-git-tag-version --allow-same-version

# git commit -am "release: v$V"
# git tag "v$V"
