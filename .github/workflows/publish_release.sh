#!/usr/bin/env bash
set -euxo pipefail

# Set Git author information
git config --global user.email "github-actions[bot]@users.noreply.github.com"
git config --global user.name "$GITHUB_REPOSITORY github-actions[bot]"

# Clone our F-Droid git repository
cd "$(mktemp -d)"
git clone --depth 1 "git@github.com:${GITHUB_REPOSITORY_OWNER}/${FDROID_DEPLOY_REPO}.git" fdroid
cd fdroid/fdroid

# Add the built APK to the signed directory and commit
mkdir -p ./unsigned
cp "$GITHUB_WORKSPACE/$APK_PATH" ./unsigned/

git add ./unsigned
git commit -m"chore: Added ${APK_NAME} through CI"

# Force push to the unsigned branch
git push origin main