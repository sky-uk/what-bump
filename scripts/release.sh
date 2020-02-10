#!/usr/bin/env sh
set -e
LATEST_VERSION=$(git tag | grep -E '[0-9]+.[0-9]+.[0-9]+.*' | sort -rV | head -1)
NEW_VERSION=$(what-bump --from "$LATEST_VERSION" "$LATEST_VERSION")
sed -i "" "s/0.0.0-UNRELEASED/$NEW_VERSION/g" Cargo.toml
cargo check
git commit -a -m "chore: release version $NEW_VERSION"
git tag -a "$NEW_VERSION" -m "Release $NEW_VERSION"
git push origin "$NEW_VERSION" develop
git checkout HEAD^ -- Cargo.toml Cargo.lock
git commit -a -m "chore: restore UNRELEASED version"
git push origin develop