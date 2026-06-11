#!/usr/bin/env bash
set -euxo pipefail

DEBIAN_FRONTEND=noninteractive npx @playwright/test@1.60.0 install-deps

# Change ownership to wallet
mkdir -p .cache # rosetta already creates cache directory
chown -R wallet:wallet .cache .npm

# Preinstall browsers
sudo -E -H -u wallet -- sh -c 'cd $HOME && npx @playwright/test@1.60.0 install chromium webkit'

rm -rf ~/.npm
