#!/usr/bin/env bash
set -euxo pipefail

DEBIAN_FRONTEND=noninteractive npx @playwright/test@1.56.1 install-deps
rm -rf ~/.npm
