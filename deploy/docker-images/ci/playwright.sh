#!/usr/bin/env bash
set -euxo pipefail

DEBIAN_FRONTEND=noninteractive npx @playwright/test@1.60.0 install-deps

rm -rf ~/.npm
