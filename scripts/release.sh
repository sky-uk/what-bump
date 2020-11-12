#!/usr/bin/env sh
set -ex

# 1. Versioning
LATEST_VERSION=$(git tag | grep -E '[0-9]+.[0-9]+.[0-9]+.*' | sort -rV | head -1)
NEW_VERSION=$(what-bump --from "$LATEST_VERSION" "$LATEST_VERSION" --changelog CHANGELOG.md --overwrite)
sed -i -e "s/0.0.0-UNRELEASED/$NEW_VERSION/g" Cargo.toml

# 2. Commit, Tag, and Create Release
cargo check
git commit -a -m "chore: release version $NEW_VERSION"
git tag -a "$NEW_VERSION" -m "Release $NEW_VERSION"
git push origin "$NEW_VERSION" develop

#3. Publish to crates.io
cargo publish

#4. Publish to dockerhub
IMAGE_NAME=albx79/what-bump:"$NEW_VERSION"
docker build . -t "$IMAGE_NAME"
docker push "$IMAGE_NAME"

#5. Publish artifact to github
id=$(docker create "$IMAGE_NAME")
docker cp $id:/what-bump what-bump
docker rm -v $id
ghr $NEW_VERSION what-bump
rm what-bump

#6. Restore snapshot version
git checkout HEAD^ -- Cargo.toml Cargo.lock
git commit -a -m "chore: restore UNRELEASED version"
git push origin develop
