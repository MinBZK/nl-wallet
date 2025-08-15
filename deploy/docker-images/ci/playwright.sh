#!/usr/bin/env bash
set -euxo pipefail

DEBIAN_FRONTEND=noninteractive npx @playwright/test@1.55.0-alpha-2025-08-11 install-deps
rm -rf ~/.npm
