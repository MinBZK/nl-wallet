#!/usr/bin/env bash

set -euo pipefail

SCRIPTS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"

source "${SCRIPTS_DIR}/utils.sh"

# Check if cargo-hack is installed
have cargo-hack

# Determine if we are in a subdirectory of wallet_core
CURRENT_DIR="$(pwd -P)"
WALLET_CORE_DIR="${BASE_DIR}/wallet_core"

if [[ "$CURRENT_DIR" == "$WALLET_CORE_DIR"* ]]; then
    # We are already inside wallet_core or a subdirectory
    TARGET_DIR="$CURRENT_DIR"
else
    # We are outside wallet_core, default to wallet_core root
    TARGET_DIR="$WALLET_CORE_DIR"
fi

cd "$TARGET_DIR"

echo -e "${INFO}Running cargo hack check --feature-powerset in ${TARGET_DIR}...${NC}"
cargo hack check --feature-powerset --no-dev-deps
