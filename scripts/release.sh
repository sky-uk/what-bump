#!/usr/bin/env sh
set -ex

# 1. Versioning
LATEST_VERSION=$(git tag | grep -E '[0-9]+.[0-9]+.[0-9]+.*' | sort -rV | head -1)
NEW_VERSION=$(what-bump --from "$LATEST_VERSION" "$LATEST_VERSION" --changelog CHANGELOG.md --overwrite)
sed -i "" "s/0.0.0-UNRELEASED/$NEW_VERSION/g" Cargo.toml

# 2. Commit and Tag
cargo check
git commit -a -m "chore: release version $NEW_VERSION"
git tag -a "$NEW_VERSION" -m "Release $NEW_VERSION"
git push origin "$NEW_VERSION" develop

#3. Publish to crates.io
cargo publish

#4. Publish to dockerhub
docker build . -t albx79/what-bump:"$NEW_VERSION"
docker push albx79/what-bump:"$NEW_VERSION"

#5. Restore snapshot version
git checkout HEAD^ -- Cargo.toml Cargo.lock
git commit -a -m "chore: restore UNRELEASED version"
git push origin develop
