#!/usr/bin/env bash
set -euxo pipefail

npx playwright@1.50.0 install-deps
rm -rf ~/.npm
