#!/usr/bin/env bash

set -euo pipefail

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"
BASE_DIR="$(dirname "${SCRIPTS_DIR}")"

cargo test --manifest-path "$BASE_DIR/wallet_core/Cargo.toml" --locked export_bindings --features ts_rs
npx prettier --write wallet_web/lib/models/*.ts
