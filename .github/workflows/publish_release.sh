#!/usr/bin/env bash
set -euxo pipefail

. ./.github/workflows/calculate_metadata.sh

# Set Git author information
git config --global user.email "github-actions[bot]@users.noreply.github.com"
git config --global user.name "$GITHUB_REPOSITORY github-actions[bot]"

# Clone our F-Droid git repository
cd "$(mktemp -d)"
git clone --depth 1 "git@github.com:${GITHUB_REPOSITORY_OWNER}/${FDROID_DEPLOY_REPO}.git" fdroid
cd ./fdroid

# Add the demo qrs to a version specific directory
qr_path="./demo/${release_channel}-${release_version}"
mkdir -p "$qr_path"

cp "$GITHUB_WORKSPACE/demo/qrs.md" "$qr_path/"
cp "$GITHUB_WORKSPACE/demo/qrs.json" "$qr_path/"
ln -s "../renderer.html" "$qr_path/index.html"

git add "$qr_path"

# Add the built APK to the signed directory and commit
cd ./fdroid
mkdir -p ./unsigned
cp "$GITHUB_WORKSPACE/$apk_path" ./unsigned/

git add ./unsigned
git commit -m"chore: Added '${apk_name}' through CI"

# Push directly to main
git push origin main